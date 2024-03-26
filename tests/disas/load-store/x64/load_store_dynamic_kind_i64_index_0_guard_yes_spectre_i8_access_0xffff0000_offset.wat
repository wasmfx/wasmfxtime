;;! target = "x86_64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -W memory64 -O static-memory-maximum-size=0 -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i64 1)

  (func (export "do_store") (param i64 i32)
    local.get 0
    local.get 1
    i32.store8 offset=0xffff0000)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load8_u offset=0xffff0000))

;; wasm[0]::function[0]:
;;    0: pushq   %rbp
;;    1: movq    %rsp, %rbp
;;    4: movq    %rdx, %rax
;;    7: addq    0x2a(%rip), %rax
;;    e: jb      0x36
;;   14: movq    0x58(%rdi), %r9
;;   18: xorq    %r8, %r8
;;   1b: addq    0x50(%rdi), %rdx
;;   1f: movl    $0xffff0000, %r10d
;;   25: addq    %r10, %rdx
;;   28: cmpq    %r9, %rax
;;   2b: cmovaq  %r8, %rdx
;;   2f: movb    %cl, (%rdx)
;;   31: movq    %rbp, %rsp
;;   34: popq    %rbp
;;   35: retq
;;   36: ud2
;;   38: addl    %eax, (%rax)
;;
;; wasm[0]::function[1]:
;;   40: pushq   %rbp
;;   41: movq    %rsp, %rbp
;;   44: movq    %rdx, %rax
;;   47: addq    0x32(%rip), %rax
;;   4e: jb      0x78
;;   54: movq    0x58(%rdi), %r8
;;   58: xorq    %rcx, %rcx
;;   5b: addq    0x50(%rdi), %rdx
;;   5f: movl    $0xffff0000, %r9d
;;   65: addq    %r9, %rdx
;;   68: cmpq    %r8, %rax
;;   6b: cmovaq  %rcx, %rdx
;;   6f: movzbq  (%rdx), %rax
;;   73: movq    %rbp, %rsp
;;   76: popq    %rbp
;;   77: retq
;;   78: ud2
;;   7a: addb    %al, (%rax)
;;   7c: addb    %al, (%rax)
;;   7e: addb    %al, (%rax)
;;   80: addl    %eax, (%rax)
