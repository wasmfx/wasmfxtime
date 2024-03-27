;;! target = "x86_64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -O static-memory-forced -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i32 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store8 offset=0xffff0000)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load8_u offset=0xffff0000))

;; wasm[0]::function[0]:
;;    0: pushq   %rbp
;;    1: movq    %rsp, %rbp
;;    4: movl    %edx, %eax
;;    6: xorq    %rdx, %rdx
;;    9: movq    %rax, %r8
;;    c: addq    0x50(%rdi), %r8
;;   10: movl    $0xffff0000, %edi
;;   15: leaq    (%r8, %rdi), %rsi
;;   19: cmpq    $0xffff, %rax
;;   20: cmovaq  %rdx, %rsi
;;   24: movb    %cl, (%rsi)
;;   26: movq    %rbp, %rsp
;;   29: popq    %rbp
;;   2a: retq
;;
;; wasm[0]::function[1]:
;;   30: pushq   %rbp
;;   31: movq    %rsp, %rbp
;;   34: movl    %edx, %eax
;;   36: xorq    %rcx, %rcx
;;   39: movq    %rax, %rdx
;;   3c: addq    0x50(%rdi), %rdx
;;   40: movl    $0xffff0000, %edi
;;   45: leaq    (%rdx, %rdi), %rsi
;;   49: cmpq    $0xffff, %rax
;;   50: cmovaq  %rcx, %rsi
;;   54: movzbq  (%rsi), %rax
;;   58: movq    %rbp, %rsp
;;   5b: popq    %rbp
;;   5c: retq
