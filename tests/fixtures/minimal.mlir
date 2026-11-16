module {
  llvm.func @main() -> i32 {
    %c = llvm.mlir.constant(42 : i32) : i32
    llvm.return %c : i32
  }
}
