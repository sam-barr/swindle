#![allow(dead_code)]
use crate::ast::*;
use crate::precodegen::*;
use crate::typechecker::*;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate::*;
use std::ptr;

const LLVM_FALSE: LLVMBool = 0;
const LLVM_TRUE: LLVMBool = 1;

macro_rules! nm {
    ($name:expr) => {
        $name.as_ptr() as *const _
    };
}

struct Builder {
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
    variables: Vec<LLVMValueRef>,
    strings: Vec<LLVMValueRef>,
    fmt_int: LLVMValueRef,
    fmt_string: LLVMValueRef,
    fmt_true: LLVMValueRef,
    fmt_false: LLVMValueRef,
    fmt_unit: LLVMValueRef,
    fmt_newline: LLVMValueRef,
    printf: LLVMValueRef,
    int64_ty: LLVMTypeRef,
    int1_ty: LLVMTypeRef,
    string_ty: LLVMTypeRef,
    main_fn: LLVMValueRef,
    end: LLVMBasicBlockRef,
    tracker: LLVMValueRef,
    break_bb: LLVMBasicBlockRef,
    continue_bb: LLVMBasicBlockRef,
}

impl Builder {
    fn new() -> Self {
        unsafe {
            let context = LLVMContextCreate();
            let builder = LLVMCreateBuilderInContext(context);
            let module = LLVMModuleCreateWithNameInContext(nm!(b"main\0"), context);
            let variables = Vec::new();
            let strings = Vec::new();

            // NOTE: not sure if this is good
            let void = LLVMVoidTypeInContext(context);
            let function_type = LLVMFunctionType(void, ptr::null_mut(), 0, 0);
            let main_fn = LLVMAddFunction(module, nm!(b"main\0"), function_type);
            let start = LLVMAppendBasicBlockInContext(context, main_fn, nm!(b"entry\0"));

            // this is a hack, since AFAIK you you can't insert after a basic block
            // so I keep a block at the end, and then delete it after compilation
            let end = LLVMAppendBasicBlockInContext(context, main_fn, nm!(b"return\0"));
            LLVMPositionBuilderAtEnd(builder, end);
            LLVMBuildRetVoid(builder);

            LLVMPositionBuilderAtEnd(builder, start);
            // NOTE end not good

            let fmt_int = LLVMBuildGlobalStringPtr(builder, nm!(b"%ld\0"), nm!(b"fmt_int\0"));
            let fmt_string = LLVMBuildGlobalStringPtr(builder, nm!(b"%s\0"), nm!(b"fmt_string\0"));
            let fmt_true = LLVMBuildGlobalStringPtr(builder, nm!(b"true\0"), nm!(b"fmt_true\0"));
            let fmt_false = LLVMBuildGlobalStringPtr(builder, nm!(b"false\0"), nm!(b"fmt_false\0"));
            let fmt_unit = LLVMBuildGlobalStringPtr(builder, nm!(b"()\0"), nm!(b"fmt_unit\0"));
            let fmt_newline =
                LLVMBuildGlobalStringPtr(builder, nm!(b"\n\0"), nm!(b"fmt_newline\0"));
            let printf_ty = LLVMFunctionType(
                LLVMInt64TypeInContext(context),
                [LLVMPointerType(LLVMInt8TypeInContext(context), 0)].as_mut_ptr(),
                1,
                LLVM_TRUE,
            );
            let printf = LLVMAddFunction(module, nm!(b"printf\0"), printf_ty);

            let int64_ty = LLVMInt64TypeInContext(context);
            let int1_ty = LLVMInt1TypeInContext(context);
            let string_ty = LLVMPointerType(LLVMInt8TypeInContext(context), 0);

            let tracker = ptr::null_mut();
            let break_bb = ptr::null_mut();
            let continue_bb = ptr::null_mut();

            Builder {
                context,
                builder,
                module,
                variables,
                strings,
                fmt_int,
                fmt_string,
                fmt_true,
                fmt_false,
                fmt_unit,
                fmt_newline,
                printf,
                int64_ty,
                int1_ty,
                string_ty,
                main_fn,
                end,
                tracker,
                break_bb,
                continue_bb,
            }
        }
    }

    unsafe fn declare_variable(&mut self, typ: SwindleType) {
        let typ = match typ {
            SwindleType::Int => self.int64_ty,
            SwindleType::Bool => self.int1_ty,
            SwindleType::Unit => self.int1_ty,
            SwindleType::String => self.string_ty,
        };
        self.variables
            .push(LLVMBuildAlloca(self.builder, typ, nm!(b"var\0")));
    }

    unsafe fn unit(&self) -> LLVMValueRef {
        LLVMConstInt(self.int1_ty, 0, LLVM_FALSE)
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
        for typ in var_info {
            builder.declare_variable(typ);
        }

        for mut string in strings {
            string.push('\0');
            builder.strings.push(LLVMBuildGlobalStringPtr(
                builder.builder,
                string.as_ptr() as *const _,
                nm!(b"str\0"),
            ));
        }

        for tagged_stmt in program.statements {
            cg_statement(&mut builder, tagged_stmt.statement);
        }
        LLVMBuildRetVoid(builder.builder);
        LLVMDeleteBasicBlock(builder.end);
        LLVMDumpModule(builder.module);
    }
}

unsafe fn cg_statement(builder: &mut Builder, statement: Statement<PCG>) -> LLVMValueRef {
    match statement {
        Statement::Declare(_, id, expression) => LLVMBuildStore(
            builder.builder,
            cg_expression(builder, *expression),
            builder.variables[id],
        ),
        Statement::Write(ty, newline, expression) => {
            match ty {
                SwindleType::Int => {
                    LLVMBuildCall(
                        builder.builder,
                        builder.printf,
                        [builder.fmt_int, cg_expression(builder, *expression)].as_mut_ptr(),
                        2,
                        nm!(b"tmp\0"),
                    );
                }
                SwindleType::Bool => {
                    let fmt = LLVMBuildSelect(
                        builder.builder,
                        cg_expression(builder, *expression),
                        builder.fmt_true,
                        builder.fmt_false,
                        nm!(b"boolfmt\0"),
                    );
                    LLVMBuildCall(
                        builder.builder,
                        builder.printf,
                        [fmt].as_mut_ptr(),
                        1,
                        nm!(b"tmp\0"),
                    );
                }
                SwindleType::String => {
                    LLVMBuildCall(
                        builder.builder,
                        builder.printf,
                        [builder.fmt_string, cg_expression(builder, *expression)].as_mut_ptr(),
                        2,
                        nm!(b"tmp\0"),
                    );
                }
                SwindleType::Unit => {
                    LLVMBuildCall(
                        builder.builder,
                        builder.printf,
                        [builder.fmt_unit, cg_expression(builder, *expression)].as_mut_ptr(),
                        2,
                        nm!(b"tmp\0"),
                    );
                }
            }

            if newline {
                LLVMBuildCall(
                    builder.builder,
                    builder.printf,
                    [builder.fmt_newline].as_mut_ptr(),
                    1,
                    nm!(b"tmp\0"),
                );
            }

            builder.unit()
        }
        Statement::Break => {
            LLVMBuildBr(builder.builder, builder.break_bb);
            builder.unit()
        }
        Statement::Continue => {
            LLVMBuildStore(
                builder.builder,
                LLVMConstInt(builder.int1_ty, 0, LLVM_FALSE),
                builder.tracker,
            );
            LLVMBuildBr(builder.builder, builder.continue_bb);
            builder.unit()
        }
        Statement::Expression(expression) => cg_expression(builder, *expression),
    }
}

unsafe fn cg_expression(builder: &mut Builder, expression: Expression<PCG>) -> LLVMValueRef {
    match expression {
        Expression::Assign(id, expression) => {
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
            nm!(b"or\0"),
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
            nm!(b"and\0"),
        ),
        AndExp::CompExp(compexp) => cg_compexp(builder, *compexp),
    }
}

unsafe fn cg_compexp(builder: &mut Builder, compexp: CompExp<PCG>) -> LLVMValueRef {
    match compexp {
        // NOTE: this won't cover string equality
        CompExp::Comp(op, addexp1, addexp2) => {
            let addexp1 = cg_addexp(builder, *addexp1);
            let addexp2 = cg_addexp(builder, *addexp2);
            let (pred, name) = match op {
                CompOp::Leq => (LLVMIntSLE, nm!(b"leq\0")),
                CompOp::Lt => (LLVMIntSLT, nm!(b"lt\0")),
                CompOp::Eq => (LLVMIntEQ, nm!(b"eq\0")),
                CompOp::Neq => (LLVMIntNE, nm!(b"eq\0")),
                CompOp::Gt => (LLVMIntSGT, nm!(b"gt\0")),
                CompOp::Geq => (LLVMIntSGE, nm!(b"geq\0")),
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
                AddOp::Sum => LLVMBuildAdd(builder.builder, mulexp, addexp, nm!(b"sum\0")),
                AddOp::Difference => {
                    LLVMBuildSub(builder.builder, mulexp, addexp, nm!(b"difference\0"))
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
                MulOp::Product => LLVMBuildMul(builder.builder, unary, mulexp, nm!(b"product\0")),
                MulOp::Quotient => {
                    LLVMBuildSDiv(builder.builder, unary, mulexp, nm!(b"quotient\0"))
                }
                MulOp::Remainder => {
                    LLVMBuildSRem(builder.builder, unary, mulexp, nm!(b"remainder\0"))
                }
            }
        }
        MulExp::Unary(unary) => cg_unary(builder, *unary),
    }
}

unsafe fn cg_unary(builder: &mut Builder, unary: Unary<PCG>) -> LLVMValueRef {
    match unary {
        Unary::Negate(unary) => {
            LLVMBuildNeg(builder.builder, cg_unary(builder, *unary), nm!(b"negate\0"))
        }
        Unary::Not(unary) => {
            LLVMBuildNot(builder.builder, cg_unary(builder, *unary), nm!(b"not\0"))
        }
        Unary::Primary(primary) => cg_primary(builder, *primary),
    }
}

unsafe fn cg_primary(builder: &mut Builder, primary: Primary<PCG>) -> LLVMValueRef {
    match primary {
        Primary::Paren(e) => cg_expression(builder, *e),
        Primary::IntLit(n) => LLVMConstInt(builder.int64_ty, n, LLVM_TRUE),
        Primary::StringLit(id) => builder.strings[id],
        Primary::BoolLit(b) => LLVMConstInt(builder.int1_ty, if b { 1 } else { 0 }, LLVM_FALSE),
        Primary::Variable(id) => {
            LLVMBuildLoad(builder.builder, builder.variables[id], nm!(b"variable\0"))
        }
        Primary::IfExp(ifexp) => cg_ifexp(builder, ifexp),
        Primary::WhileExp(whileexp) => cg_whileexp(builder, whileexp),
        Primary::StatementExp(body) => cg_body(builder, body),
        Primary::Unit => builder.unit(),
    }
}

unsafe fn cg_whileexp(builder: &mut Builder, whileexp: WhileExp<PCG>) -> LLVMValueRef {
    let typ = match whileexp.tag {
        SwindleType::Int => builder.int64_ty,
        SwindleType::Bool => builder.int1_ty,
        SwindleType::Unit => builder.int1_ty,
        SwindleType::String => builder.string_ty,
    };

    let old_tracker = builder.tracker;
    let old_break_bb = builder.break_bb;
    let old_continue_bb = builder.break_bb;

    //setup blocks and variables
    let current_block = LLVMGetInsertBlock(builder.builder);
    let next_block = LLVMGetNextBasicBlock(current_block);
    let while_result = LLVMBuildAlloca(builder.builder, typ, nm!(b"while_result\0"));
    // if while_tracker: skip else
    let while_tracker = LLVMBuildAlloca(builder.builder, builder.int1_ty, nm!(b"while_tracker\0"));
    LLVMBuildStore(
        builder.builder,
        LLVMConstInt(builder.int1_ty, 0, LLVM_FALSE),
        while_tracker,
    );
    let start = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!(b"start\0"));
    let then = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!(b"then\0"));
    let otherwise = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!(b"otherwise\0"));
    let els = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!(b"els\0"));
    let finally = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!(b"finally\0"));

    builder.tracker = while_tracker;
    builder.break_bb = els;
    builder.continue_bb = start;

    LLVMPositionBuilderAtEnd(builder.builder, current_block);
    LLVMBuildBr(builder.builder, start);
    LLVMPositionBuilderAtEnd(builder.builder, start);

    let cond = cg_expression(builder, *whileexp.cond);
    // while_tracker = while_tracker or cond
    LLVMBuildStore(
        builder.builder,
        LLVMBuildOr(
            builder.builder,
            cond,
            LLVMBuildLoad(builder.builder, while_tracker, nm!(b"tmp\0")),
            nm!(b"tmp\0"),
        ),
        while_tracker,
    );
    LLVMBuildCondBr(builder.builder, cond, then, otherwise);
    LLVMPositionBuilderAtEnd(builder.builder, then);
    LLVMBuildStore(
        builder.builder,
        cg_body(builder, whileexp.body),
        while_result,
    );
    LLVMBuildBr(builder.builder, start);

    LLVMPositionBuilderAtEnd(builder.builder, otherwise);
    LLVMBuildCondBr(
        builder.builder,
        LLVMBuildLoad(builder.builder, while_tracker, nm!(b"tmp\0")),
        finally,
        els,
    );

    LLVMPositionBuilderAtEnd(builder.builder, els);
    LLVMBuildStore(
        builder.builder,
        cg_body(builder, whileexp.els),
        while_result,
    );
    LLVMBuildBr(builder.builder, finally);

    builder.tracker = old_tracker;
    builder.break_bb = old_break_bb;
    builder.continue_bb = old_continue_bb;
    LLVMPositionBuilderAtEnd(builder.builder, finally);
    LLVMBuildLoad(builder.builder, while_result, nm!(b"whileexp\0"))
}

unsafe fn cg_ifexp(builder: &mut Builder, ifexp: IfExp<PCG>) -> LLVMValueRef {
    let typ = match ifexp.tag {
        SwindleType::Int => builder.int64_ty,
        SwindleType::Bool => builder.int1_ty,
        SwindleType::Unit => builder.int1_ty,
        SwindleType::String => builder.string_ty,
    };
    let current_block = LLVMGetInsertBlock(builder.builder);
    let next_block = LLVMGetNextBasicBlock(current_block);
    let if_result = LLVMBuildAlloca(builder.builder, typ, nm!(b"if_result\0"));
    let then = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!(b"then\0"));
    let mut otherwise =
        LLVMInsertBasicBlockInContext(builder.context, next_block, nm!(b"otherwise\0"));
    let finally = LLVMInsertBasicBlockInContext(builder.context, next_block, nm!(b"finally\0"));
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
        let new_then = LLVMInsertBasicBlockInContext(builder.context, finally, nm!(b"then\0"));
        let new_otherwise =
            LLVMInsertBasicBlockInContext(builder.context, finally, nm!(b"otherwise\0"));
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
    LLVMBuildLoad(builder.builder, if_result, nm!(b"ifexp\0"))
}

unsafe fn cg_body(builder: &mut Builder, body: Body<PCG>) -> LLVMValueRef {
    let mut value = builder.unit();
    for stmt in body.statements {
        value = cg_statement(builder, stmt);
    }
    value
}
