;;! target = "x86_64"
;;! test = "winch"

(module
    (func (param i64) (result i64)
        (local.get 0)
        (i64.ctz)
    )
)
;; wasm[0]::function[0]:
;;       pushq   %rbp
;;       movq    %rsp, %rbp
;;       movq    8(%rdi), %r11
;;       movq    0x10(%r11), %r11
;;       addq    $0x20, %r11
;;       cmpq    %rsp, %r11
;;       ja      0x52
;;   1c: movq    %rdi, %r14
;;       subq    $0x20, %rsp
;;       movq    %rdi, 0x18(%rsp)
;;       movq    %rsi, 0x10(%rsp)
;;       movq    %rdx, 8(%rsp)
;;       movq    8(%rsp), %rax
;;       bsfq    %rax, %rax
;;       movl    $0, %r11d
;;       sete    %r11b
;;       shlq    $6, %r11
;;       addq    %r11, %rax
;;       addq    $0x20, %rsp
;;       popq    %rbp
;;       retq
;;   52: ud2
