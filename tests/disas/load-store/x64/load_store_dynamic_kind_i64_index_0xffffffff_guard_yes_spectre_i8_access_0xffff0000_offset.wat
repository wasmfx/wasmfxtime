;;! target = "x86_64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -W memory64 -O static-memory-maximum-size=0 -O static-memory-guard-size=4294967295 -O dynamic-memory-guard-size=4294967295"

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

;; function u0:0:
;;   pushq   %rbp
;;   unwind PushFrameRegs { offset_upward_to_caller_sp: 16 }
;;   movq    %rsp, %rbp
;;   unwind DefineNewFrame { offset_upward_to_caller_sp: 16, offset_downward_to_clobbers: 0 }
;; block0:
;;   movq    88(%rdi), %rax
;;   xorq    %r8, %r8, %r8
;;   movq    %rdx, %r9
;;   addq    %r9, 80(%rdi), %r9
;;   movl    $-65536, %edi
;;   lea     0(%r9,%rdi,1), %rsi
;;   cmpq    %rax, %rdx
;;   cmovnbeq %r8, %rsi, %rsi
;;   movb    %cl, 0(%rsi)
;;   jmp     label1
;; block1:
;;   movq    %rbp, %rsp
;;   popq    %rbp
;;   ret
;;
;; function u0:1:
;;   pushq   %rbp
;;   unwind PushFrameRegs { offset_upward_to_caller_sp: 16 }
;;   movq    %rsp, %rbp
;;   unwind DefineNewFrame { offset_upward_to_caller_sp: 16, offset_downward_to_clobbers: 0 }
;; block0:
;;   movq    88(%rdi), %rax
;;   xorq    %rcx, %rcx, %rcx
;;   movq    %rdx, %r8
;;   addq    %r8, 80(%rdi), %r8
;;   movl    $-65536, %edi
;;   lea     0(%r8,%rdi,1), %rsi
;;   cmpq    %rax, %rdx
;;   cmovnbeq %rcx, %rsi, %rsi
;;   movzbq  0(%rsi), %rax
;;   jmp     label1
;; block1:
;;   movq    %rbp, %rsp
;;   popq    %rbp
;;   ret
