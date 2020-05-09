#![allow(dead_code)]
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use std::ptr;

struct Builder {
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
}

impl Builder {
    fn new() -> Self {
        unsafe {
            let context = LLVMContextCreate();
            let builder = LLVMCreateBuilderInContext(context);
            let module = LLVMModuleCreateWithName(b"test\0".as_ptr() as *const _);
            Builder {
                context,
                builder,
                module,
            }
        }
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

pub unsafe fn build_llvm(_strings: &[String], num_variables: usize) {
    let builder = Builder::new();

    let int64 = LLVMInt64TypeInContext(builder.context);
    let void = LLVMVoidTypeInContext(builder.context);
    let function_type = LLVMFunctionType(void, ptr::null_mut(), 0, 0);
    let function = LLVMAddFunction(
        builder.module,
        b"main\0".as_ptr() as *const _,
        function_type,
    );

    let bb = LLVMAppendBasicBlockInContext(
        builder.context,
        function,
        b"theEntryPoint\0".as_ptr() as *const _,
    );

    LLVMPositionBuilderAtEnd(builder.builder, bb);

    let mut variables = vec![ptr::null_mut(); num_variables];
    for i in 0..num_variables {
        variables[i] = LLVMBuildAlloca(builder.builder, int64, b"var\0".as_ptr() as *const _);
    }

    LLVMDumpModule(builder.module);
}
