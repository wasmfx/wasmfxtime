;;! target = "x86_64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -W memory64 -O static-memory-forced -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i64 1)

  (func (export "do_store") (param i64 i32)
    local.get 0
    local.get 1
    i32.store offset=0x1000)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load offset=0x1000))

;; wasm[0]::function[0]:
;;    0: pushq   %rbp
;;    1: movq    %rsp, %rbp
;;    4: xorq    %r11, %r11
;;    7: movq    0x50(%rdi), %rsi
;;    b: leaq    0x1000(%rsi, %rdx), %r10
;;   13: cmpq    0xe(%rip), %rdx
;;   1a: cmovaq  %r11, %r10
;;   1e: movl    %ecx, (%r10)
;;   21: movq    %rbp, %rsp
;;   24: popq    %rbp
;;   25: retq
;;   26: addb    %al, (%rax)
;;   28: cld
;;   29: outl    %eax, %dx
;;
;; wasm[0]::function[1]:
;;   30: pushq   %rbp
;;   31: movq    %rsp, %rbp
;;   34: xorq    %r11, %r11
;;   37: movq    0x50(%rdi), %rsi
;;   3b: leaq    0x1000(%rsi, %rdx), %r10
;;   43: cmpq    0xe(%rip), %rdx
;;   4a: cmovaq  %r11, %r10
;;   4e: movl    (%r10), %eax
;;   51: movq    %rbp, %rsp
;;   54: popq    %rbp
;;   55: retq
;;   56: addb    %al, (%rax)
;;   58: cld
;;   59: outl    %eax, %dx
