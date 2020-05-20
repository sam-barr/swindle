use crate::ast::*;
use crate::precodegen::*;
use crate::typechecker::*;
use llvm_sys::core::*;
use llvm_sys::ir_reader::*;
use llvm_sys::linker::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate::*;
use std::ptr;

const LLVM_FALSE: LLVMBool = 0;
const LLVM_TRUE: LLVMBool = 1;

const RTS_SOURCES: [&[u8]; 3] = [
    include_bytes!("../rts/io.ll"),
    include_bytes!("../rts/rc.ll"),
    include_bytes!("../rts/strings.ll"),
];

macro_rules! nm {
    ($name:expr) => {
        concat!($name, '\0').as_ptr() as *const i8
    };
}

unsafe fn load_rts_source(context: LLVMContextRef, code: &[u8]) -> LLVMModuleRef {
    let mut code = code.to_vec();
    code.push(0);
    let memory_buffer = LLVMCreateMemoryBufferWithMemoryRange(
        code.as_ptr() as *const i8,
        code.len() - 1,
        nm!(""),
        LLVM_TRUE,
    );
    let mut module = ptr::null_mut();
    LLVMParseIRInContext(context, memory_buffer, &mut module, ptr::null_mut());

    module
}

struct Builder {
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
    variables: Vec<LLVMValueRef>,
    strings: Vec<LLVMValueRef>,
    end: LLVMBasicBlockRef,
    break_bb: LLVMBasicBlockRef,
    continue_bb: LLVMBasicBlockRef,
}

impl Builder {
    fn new() -> Self {
        unsafe {
            let context = LLVMContextCreate();
            let builder = LLVMCreateBuilderInContext(context);
            let module = LLVMModuleCreateWithNameInContext(nm!("main"), context);
            let variables = Vec::new();
            let strings = Vec::new();

            for rts_source in RTS_SOURCES.iter() {
                let rts_module = load_rts_source(context, rts_source);
                LLVMLinkModules2(module, rts_module);
            }

            // NOTE: not sure if this is good
            let void = LLVMVoidTypeInContext(context);
            let function_type = LLVMFunctionType(void, ptr::null_mut(), 0, 0);
            let main_fn = LLVMAddFunction(module, nm!("main"), function_type);
            let start = LLVMAppendBasicBlockInContext(context, main_fn, nm!("entry"));

            // this is a hack, since AFAIK you you can't insert after a basic block
            // so I keep a block at the end, and then delete it after compilation
            let end = LLVMAppendBasicBlockInContext(context, main_fn, nm!("return"));
            LLVMPositionBuilderAtEnd(builder, end);
            LLVMBuildRetVoid(builder);

            LLVMPositionBuilderAtEnd(builder, start);
            // NOTE end not good

            let break_bb = ptr::null_mut();
            let continue_bb = ptr::null_mut();

            Builder {
                context,
                builder,
                module,
                variables,
                strings,
                end,
                break_bb,
                continue_bb,
            }
        }
    }

    unsafe fn declare_variable(&mut self, typ: &SwindleType) {
        let llvm_type = match typ {
            SwindleType::Int => self.int64_ty(),
            SwindleType::Bool => self.int1_ty(),
            SwindleType::Unit => self.int1_ty(),
            SwindleType::String => self.string_ty(),
        };
        let var = LLVMBuildAlloca(self.builder, llvm_type, nm!("var"));
        self.variables.push(var);
        if let SwindleType::String = typ {
            let rc = LLVMBuildAlloca(
                self.builder,
                LLVMGetTypeByName(self.module, nm!("struct.RC")),
                nm!("rc"),
            );
            LLVMBuildCall(
                self.builder,
                LLVMGetNamedFunction(self.module, nm!("uninit")),
                [rc].as_mut_ptr(),
                1,
                nm!(""),
            );
            LLVMBuildStore(self.builder, rc, var);
        }
    }

    unsafe fn add_string(&mut self, mut string: String) {
        string.push('\0');
        let string =
            LLVMBuildGlobalStringPtr(self.builder, string.as_ptr() as *const i8, nm!("str_const"));
        let rc = LLVMBuildAlloca(
            self.builder,
            LLVMGetTypeByName(self.module, nm!("struct.RC")),
            nm!("str"),
        );
        LLVMBuildCall(
            self.builder,
            LLVMGetNamedFunction(self.module, nm!("rc_string")),
            [rc, string].as_mut_ptr(),
            2,
            nm!(""),
        );
        LLVMBuildCall(
            self.builder,
            LLVMGetNamedFunction(self.module, nm!("alloc")),
            [rc].as_mut_ptr(),
            1,
            nm!(""),
        );
        self.strings.push(rc);
    }

    unsafe fn const_int(&self, n: u64) -> LLVMValueRef {
        LLVMConstInt(self.int64_ty(), n, LLVM_TRUE)
    }

    unsafe fn const_bool(&self, b: bool) -> LLVMValueRef {
        LLVMConstInt(self.int1_ty(), if b { 1 } else { 0 }, LLVM_FALSE)
    }

    unsafe fn unit(&self) -> LLVMValueRef {
        LLVMConstInt(self.int1_ty(), 0, LLVM_FALSE)
    }

    unsafe fn int64_ty(&self) -> LLVMTypeRef {
        LLVMInt64TypeInContext(self.context)
    }

    unsafe fn int1_ty(&self) -> LLVMTypeRef {
        LLVMInt1TypeInContext(self.context)
    }

    unsafe fn string_ty(&self) -> LLVMTypeRef {
        LLVMPointerType(LLVMGetTypeByName(self.module, nm!("struct.RC")), 0)
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.context);
        }
    }
}

pub fn cg_program(program: Program<PCG>, var_info: Vec<SwindleType>, strings: Vec<String>) {
    unsafe {
        let mut builder = Builder::new();
        for typ in &var_info {
            builder.declare_variable(&typ);
        }
        for string in strings {
            builder.add_string(string);
        }
        for tagged_stmt in program.statements {
            cg_tagged_statement(&mut builder, tagged_stmt);
        }
        for (idx, typ) in var_info.iter().enumerate() {
            if let SwindleType::String = typ {
                LLVMBuildCall(
                    builder.builder,
                    LLVMGetNamedFunction(builder.module, nm!("drop2")),
                    [builder.variables[idx]].as_mut_ptr(),
                    1,
                    nm!(""),
                );
            }
        }
        for idx in 0..builder.strings.len() {
            LLVMBuildCall(
                builder.builder,
                LLVMGetNamedFunction(builder.module, nm!("drop")),
                [builder.strings[idx]].as_mut_ptr(),
                1,
                nm!(""),
            );
        }
        LLVMBuildRetVoid(builder.builder);
        LLVMDeleteBasicBlock(builder.end);
        LLVMDumpModule(builder.module);
    }
}

unsafe fn cg_tagged_statement(
    builder: &mut Builder,
    tagged_stmt: TaggedStatement<PCG>,
) -> LLVMValueRef {
    let value = cg_statement(builder, tagged_stmt.statement);
    if tagged_stmt.tag {
        LLVMBuildCall(
            builder.builder,
            LLVMGetNamedFunction(builder.module, nm!("destroy_noref")),
            [value].as_mut_ptr(),
            1,
            nm!(""),
        );
    }
    value
}

unsafe fn cg_statement(builder: &mut Builder, statement: Statement<PCG>) -> LLVMValueRef {
    match statement {
        Statement::Declare(SwindleType::String, id, expression) => {
            LLVMBuildCall(
                builder.builder,
                LLVMGetNamedFunction(builder.module, nm!("drop2")),
                [builder.variables[id]].as_mut_ptr(),
                1,
                nm!(""),
            );
            let rc = LLVMBuildCall(
                builder.builder,
                LLVMGetNamedFunction(builder.module, nm!("alloc")),
                [cg_expression(builder, *expression)].as_mut_ptr(),
                1,
                nm!("rc"),
            );
            LLVMBuildStore(builder.builder, rc, builder.variables[id]);
            builder.unit()
        }
        Statement::Declare(_, id, expression) => {
            LLVMBuildStore(
                builder.builder,
                cg_expression(builder, *expression),
                builder.variables[id],
            );
            builder.unit()
        }
        Statement::Write(ty, newline, expression) => {
            let print_fn = LLVMGetNamedFunction(
                builder.module,
                match ty {
                    SwindleType::Int => nm!("print_int"),
                    SwindleType::String => nm!("print_string"),
                    SwindleType::Bool => nm!("print_bool"),
                    SwindleType::Unit => nm!("print_unit"),
                },
            );
            let expression = cg_expression(builder, *expression);
            LLVMBuildCall(
                builder.builder,
                print_fn,
                [expression, builder.const_bool(newline)].as_mut_ptr(),
                2,
                nm!(""),
            );
            builder.unit()
        }
        Statement::Break => {
            LLVMBuildBr(builder.builder, builder.break_bb);
            builder.unit()
        }
        Statement::Continue => {
            LLVMBuildBr(builder.builder, builder.continue_bb);
            builder.unit()
        }
        Statement::Expression(expression) => cg_expression(builder, *expression),
    }
}

unsafe fn cg_expression(builder: &mut Builder, expression: Expression<PCG>) -> LLVMValueRef {
    match expression {
        Expression::Assign(SwindleType::String, id, expression) => {
            LLVMBuildCall(
                builder.builder,
                LLVMGetNamedFunction(builder.module, nm!("drop2")),
                [builder.variables[id]].as_mut_ptr(),
                1,
                nm!(""),
            );
            let expression = cg_expression(builder, *expression);
            let rc = LLVMBuildCall(
                builder.builder,
                LLVMGetNamedFunction(builder.module, nm!("alloc")),
                [expression].as_mut_ptr(),
                1,
                nm!("rc"),
            );
            LLVMBuildStore(builder.builder, rc, builder.variables[id]);
            expression
        }
        Expression::Assign(_, id, expression) => {
            let value = cg_expression(builder, *expression);
            LLVMBuildStore(builder.builder, value, builder.variables[id]);
            value
        }
        Expression::OrExp(orexp) => cg_orexp(builder, *orexp),
    }
}

unsafe fn cg_orexp(builder: &mut Builder, orexp: OrExp<PCG>) -> LLVMValueRef {
    match orexp {
        OrExp::Or(andexp, orexp) => LLVMBuildOr(
            builder.builder,
            cg_andexp(builder, *andexp),
            cg_orexp(builder, *orexp),
            nm!("or"),
        ),
        OrExp::AndExp(andexp) => cg_andexp(builder, *andexp),
    }
}

unsafe fn cg_andexp(builder: &mut Builder, andexp: AndExp<PCG>) -> LLVMValueRef {
    match andexp {
        AndExp::And(compexp, andexp) => LLVMBuildAnd(
            builder.builder,
            cg_compexp(builder, *compexp),
            cg_andexp(builder, *andexp),
            nm!("and"),
        ),
        AndExp::CompExp(compexp) => cg_compexp(builder, *compexp),
    }
}

unsafe fn cg_compexp(builder: &mut Builder, compexp: CompExp<PCG>) -> LLVMValueRef {
    match compexp {
        CompExp::Comp(CompOp::Eq(SwindleType::String), addexp1, addexp2) => {
            let addexp1 = cg_addexp(builder, *addexp1);
            let addexp2 = cg_addexp(builder, *addexp2);
            LLVMBuildCall(
                builder.builder,
                LLVMGetNamedFunction(builder.module, nm!("streq")),
                [addexp1, addexp2].as_mut_ptr(),
                2,
                nm!(""),
            )
        }
        CompExp::Comp(op, addexp1, addexp2) => {
            let addexp1 = cg_addexp(builder, *addexp1);
            let addexp2 = cg_addexp(builder, *addexp2);
            let (pred, name) = match op {
                CompOp::Leq => (LLVMIntSLE, nm!("leq")),
                CompOp::Lt => (LLVMIntSLT, nm!("lt")),
                CompOp::Eq(_) => (LLVMIntEQ, nm!("eq")),
            };
            LLVMBuildICmp(builder.builder, pred, addexp1, addexp2, name)
        }
        CompExp::AddExp(addexp) => cg_addexp(builder, *addexp),
    }
}

unsafe fn cg_addexp(builder: &mut Builder, addexp: AddExp<PCG>) -> LLVMValueRef {
    match addexp {
        AddExp::Add(op, mulexp, addexp) => {
            let mulexp = cg_mulexp(builder, *mulexp);
            let addexp = cg_addexp(builder, *addexp);
            match op {
                AddOp::Sum(SwindleType::String) => {
                    let rc = LLVMBuildAlloca(
                        builder.builder,
                        LLVMGetTypeByName(builder.module, nm!("struct.RC")),
                        nm!("rc"),
                    );
                    LLVMBuildCall(
                        builder.builder,
                        LLVMGetNamedFunction(builder.module, nm!("append")),
                        [rc, mulexp, addexp].as_mut_ptr(),
                        3,
                        nm!(""),
                    );
                    rc
                }
                AddOp::Sum(SwindleType::Int) => {
                    LLVMBuildAdd(builder.builder, mulexp, addexp, nm!("sum"))
                }
                AddOp::Sum(_) => panic!("this should be impossible"),
                AddOp::Difference => {
                    LLVMBuildSub(builder.builder, mulexp, addexp, nm!("difference"))
                }
            }
        }
        AddExp::MulExp(mulexp) => cg_mulexp(builder, *mulexp),
    }
}

unsafe fn cg_mulexp(builder: &mut Builder, mulexp: MulExp<PCG>) -> LLVMValueRef {
    match mulexp {
        MulExp::Mul(op, unary, mulexp) => {
            let unary = cg_unary(builder, *unary);
            let mulexp = cg_mulexp(builder, *mulexp);
            match op {
                MulOp::Product => LLVMBuildMul(builder.builder, unary, mulexp, nm!("product")),
                MulOp::Quotient => LLVMBuildSDiv(builder.builder, unary, mulexp, nm!("quotient")),
                MulOp::Remainder => LLVMBuildSRem(builder.builder, unary, mulexp, nm!("remainder")),
            }
        }
        MulExp::Unary(unary) => cg_unary(builder, *unary),
    }
}

unsafe fn cg_unary(builder: &mut Builder, unary: Unary<PCG>) -> LLVMValueRef {
    match unary {
        Unary::Negate(unary) => {
            LLVMBuildNeg(builder.builder, cg_unary(builder, *unary), nm!("negate"))
        }
        Unary::Not(unary) => LLVMBuildNot(builder.builder, cg_unary(builder, *unary), nm!("not")),
        Unary::Primary(primary) => cg_primary(builder, *primary),
    }
}

unsafe fn cg_primary(builder: &mut Builder, primary: Primary<PCG>) -> LLVMValueRef {
    match primary {
        Primary::Paren(e) => cg_expression(builder, *e),
        Primary::IntLit(n) => builder.const_int(n),
        Primary::StringLit(id) => builder.strings[id],
        Primary::BoolLit(b) => builder.const_bool(b),
        Primary::Variable(id) => {
            LLVMBuildLoad(builder.builder, builder.variables[id], nm!("variable"))
        }
        Primary::IfExp(ifexp) => cg_ifexp(builder, ifexp),
        Primary::WhileExp(whileexp) => cg_whileexp(builder, whileexp),
        Primary::StatementExp(body) => cg_body(builder, body),
        Primary::Unit => builder.unit(),
    }
}

unsafe fn cg_whileexp(builder: &mut Builder, whileexp: WhileExp<PCG>) -> LLVMValueRef {
    let old_break_bb = builder.break_bb;
    let old_continue_bb = builder.break_bb;

    //setup blocks and variables
    let current_block = LLVMGetInsertBlock(builder.builder);
    let next_block = LLVMGetNextBasicBlock(current_block);
    let start = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!("start"));
    let then = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!("then"));
    let otherwise = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!("otherwise"));

    builder.break_bb = otherwise;
    builder.continue_bb = start;

    LLVMPositionBuilderAtEnd(builder.builder, current_block);
    LLVMBuildBr(builder.builder, start);
    LLVMPositionBuilderAtEnd(builder.builder, start);

    let cond = cg_expression(builder, *whileexp.cond);
    LLVMBuildCondBr(builder.builder, cond, then, otherwise);
    LLVMPositionBuilderAtEnd(builder.builder, then);
    cg_body(builder, whileexp.body);
    LLVMBuildBr(builder.builder, start);

    LLVMPositionBuilderAtEnd(builder.builder, otherwise);

    builder.break_bb = old_break_bb;
    builder.continue_bb = old_continue_bb;
    builder.unit()
}

unsafe fn cg_ifexp(builder: &mut Builder, ifexp: IfExp<PCG>) -> LLVMValueRef {
    let typ = match ifexp.tag {
        SwindleType::Int => builder.int64_ty(),
        SwindleType::Bool => builder.int1_ty(),
        SwindleType::Unit => builder.int1_ty(),
        SwindleType::String => builder.string_ty(),
    };
    let current_block = LLVMGetInsertBlock(builder.builder);
    let next_block = LLVMGetNextBasicBlock(current_block);
    let if_result = LLVMBuildAlloca(builder.builder, typ, nm!("if_result"));
    let then = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!("then"));
    let mut otherwise =
        LLVMInsertBasicBlockInContext(builder.context, next_block, nm!("otherwise"));
    let finally = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!("finally"));
    LLVMPositionBuilderAtEnd(builder.builder, current_block);
    LLVMBuildCondBr(
        builder.builder,
        cg_expression(builder, *ifexp.cond),
        then,
        otherwise,
    );
    LLVMPositionBuilderAtEnd(builder.builder, then);
    LLVMBuildStore(builder.builder, cg_body(builder, ifexp.body), if_result);
    LLVMBuildBr(builder.builder, finally);

    for elif in ifexp.elifs {
        let new_then = LLVMInsertBasicBlockInContext(builder.context, finally, nm!("then"));
        let new_otherwise =
            LLVMInsertBasicBlockInContext(builder.context, finally, nm!("otherwise"));
        LLVMPositionBuilderAtEnd(builder.builder, otherwise);
        LLVMBuildCondBr(
            builder.builder,
            cg_expression(builder, *elif.cond),
            new_then,
            new_otherwise,
        );
        LLVMPositionBuilderAtEnd(builder.builder, new_then);
        LLVMBuildStore(builder.builder, cg_body(builder, elif.body), if_result);
        LLVMBuildBr(builder.builder, finally);
        otherwise = new_otherwise;
    }

    LLVMPositionBuilderAtEnd(builder.builder, otherwise);
    LLVMBuildStore(builder.builder, cg_body(builder, ifexp.els), if_result);
    LLVMBuildBr(builder.builder, finally);

    // finally, return whatever got stored in if_result
    LLVMPositionBuilderAtEnd(builder.builder, finally);
    LLVMBuildLoad(builder.builder, if_result, nm!("ifexp"))
}

unsafe fn cg_body(builder: &mut Builder, body: Body<PCG>) -> LLVMValueRef {
    let mut value = builder.unit();
    for tagged_stmt in body.statements {
        value = cg_tagged_statement(builder, tagged_stmt);
    }
    value
}
