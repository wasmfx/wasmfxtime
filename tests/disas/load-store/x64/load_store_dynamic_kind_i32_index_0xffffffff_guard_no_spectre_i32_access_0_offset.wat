;;! target = "x86_64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -O static-memory-maximum-size=0 -O static-memory-guard-size=4294967295 -O dynamic-memory-guard-size=4294967295"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i32 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store offset=0)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load offset=0))

;; wasm[0]::function[0]:
;;    0: pushq   %rbp
;;    1: movq    %rsp, %rbp
;;    4: movq    0x58(%rdi), %r11
;;    8: movl    %edx, %r10d
;;    b: cmpq    %r11, %r10
;;    e: ja      0x21
;;   14: movq    0x50(%rdi), %rsi
;;   18: movl    %ecx, (%rsi, %r10)
;;   1c: movq    %rbp, %rsp
;;   1f: popq    %rbp
;;   20: retq
;;   21: ud2
;;
;; wasm[0]::function[1]:
;;   30: pushq   %rbp
;;   31: movq    %rsp, %rbp
;;   34: movq    0x58(%rdi), %r11
;;   38: movl    %edx, %r10d
;;   3b: cmpq    %r11, %r10
;;   3e: ja      0x51
;;   44: movq    0x50(%rdi), %rsi
;;   48: movl    (%rsi, %r10), %eax
;;   4c: movq    %rbp, %rsp
;;   4f: popq    %rbp
;;   50: retq
;;   51: ud2
