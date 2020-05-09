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
}

impl Builder {
    fn new() -> Self {
        unsafe {
            let context = LLVMContextCreate();
            let builder = LLVMCreateBuilderInContext(context);
            let module = LLVMModuleCreateWithNameInContext(nm!(b"main\0"), context);
            let variables = Vec::new();

            // NOTE: not sure if this is good
            let void = LLVMVoidTypeInContext(context);
            let function_type = LLVMFunctionType(void, ptr::null_mut(), 0, 0);
            let function = LLVMAddFunction(module, nm!(b"main\0"), function_type);
            let bb = LLVMAppendBasicBlockInContext(context, function, nm!(b"entry\0"));
            LLVMPositionBuilderAtEnd(builder, bb);

            Builder {
                context,
                builder,
                module,
                variables,
            }
        }
    }

    unsafe fn declare_variable(&mut self, typ: SwindleType) {
        let typ = match typ {
            SwindleType::Int => self.int64(),
            SwindleType::Bool => self.int1(),
            SwindleType::Unit => self.int1(),
            SwindleType::String => unimplemented!(),
        };
        self.variables
            .push(LLVMBuildAlloca(self.builder, typ, nm!(b"var\0")));
    }

    unsafe fn int64(&self) -> LLVMTypeRef {
        LLVMInt64TypeInContext(self.context)
    }

    unsafe fn int1(&self) -> LLVMTypeRef {
        LLVMInt1TypeInContext(self.context)
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.context);
            drop(self.builder);
            drop(self.module);
            drop(self.context);
        }
    }
}

pub unsafe fn cg_program(program: Program<PCG>, var_info: Vec<SwindleType>) {
    let mut builder = Builder::new();
    for typ in var_info {
        builder.declare_variable(typ);
    }

    for tagged_stmt in program.statements {
        cg_statement(&mut builder, tagged_stmt.statement);
    }

    LLVMDumpModule(builder.module);
}

unsafe fn cg_statement(builder: &Builder, statement: Statement<PCG>) -> LLVMValueRef {
    match statement {
        Statement::Declare(_, id, expression) => LLVMBuildStore(
            builder.builder,
            cg_expression(builder, *expression),
            builder.variables[id],
        ),
        Statement::Write(_, _) => unimplemented!(),
        Statement::Writeln(_, _) => unimplemented!(),
        Statement::Break => unimplemented!(),
        Statement::Continue => unimplemented!(),
        Statement::Expression(expression) => cg_expression(builder, *expression),
    }
}

unsafe fn cg_expression(builder: &Builder, expression: Expression<PCG>) -> LLVMValueRef {
    match expression {
        Expression::Assign(id, expression) => {
            let value = cg_expression(builder, *expression);
            LLVMBuildStore(builder.builder, value, builder.variables[id]);
            value
        }
        Expression::OrExp(orexp) => cg_orexp(builder, *orexp),
    }
}

unsafe fn cg_orexp(builder: &Builder, orexp: OrExp<PCG>) -> LLVMValueRef {
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

unsafe fn cg_andexp(builder: &Builder, andexp: AndExp<PCG>) -> LLVMValueRef {
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

unsafe fn cg_compexp(builder: &Builder, compexp: CompExp<PCG>) -> LLVMValueRef {
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

unsafe fn cg_addexp(builder: &Builder, addexp: AddExp<PCG>) -> LLVMValueRef {
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

unsafe fn cg_mulexp(builder: &Builder, mulexp: MulExp<PCG>) -> LLVMValueRef {
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

unsafe fn cg_unary(builder: &Builder, unary: Unary<PCG>) -> LLVMValueRef {
    match unary {
        Unary::Negate(unary) => {
            LLVMBuildNeg(builder.builder, cg_unary(builder, *unary), nm!(b"negate\0"))
        }
        Unary::Not(unary) => {
            LLVMBuildNot(builder.builder, cg_unary(builder, *unary), nm!(b"not\0"))
        }
        Unary::Stringify(_) => unimplemented!(),
        Unary::Primary(primary) => cg_primary(builder, *primary),
    }
}

unsafe fn cg_primary(builder: &Builder, primary: Primary<PCG>) -> LLVMValueRef {
    match primary {
        Primary::Paren(_) => unimplemented!(),
        Primary::IntLit(n) => LLVMConstInt(builder.int64(), n, LLVM_TRUE),
        Primary::StringLit(_) => unimplemented!(),
        Primary::BoolLit(b) => LLVMConstInt(builder.int1(), if b { 1 } else { 0 }, LLVM_FALSE),
        Primary::Variable(id) => {
            LLVMBuildLoad(builder.builder, builder.variables[id], nm!(b"variable\0"))
        }
        Primary::IfExp(_) => unimplemented!(),
        Primary::WhileExp(_) => unimplemented!(),
        Primary::Unit => LLVMConstInt(builder.int1(), 0, LLVM_FALSE),
    }
}

//pub unsafe fn build_llvm(_strings: &[String], num_variables: usize) {
//    let builder = Builder::new();
//
//    let int64 = LLVMInt64TypeInContext(builder.context);
//    let void = LLVMVoidTypeInContext(builder.context);
//    let function_type = LLVMFunctionType(void, ptr::null_mut(), 0, 0);
//    let function = LLVMAddFunction(
//        builder.module,
//        b"main\0".as_ptr() as *const _,
//        function_type,
//    );
//
//    let bb = LLVMAppendBasicBlockInContext(
//        builder.context,
//        function,
//        b"theEntryPoint\0".as_ptr() as *const _,
//    );
//
//    LLVMPositionBuilderAtEnd(builder.builder, bb);
//
//    let mut variables = vec![ptr::null_mut(); num_variables];
//    for i in 0..num_variables {
//        variables[i] = LLVMBuildAlloca(builder.builder, int64, b"var\0".as_ptr() as *const _);
//    }
//
//    LLVMDumpModule(builder.module);
//}
