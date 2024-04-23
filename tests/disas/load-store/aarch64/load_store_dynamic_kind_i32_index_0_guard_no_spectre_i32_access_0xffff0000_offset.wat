;;! target = "aarch64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -O static-memory-maximum-size=0 -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i32 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store offset=0xffff0000)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load offset=0xffff0000))

;; wasm[0]::function[0]:
;;       stp     x29, x30, [sp, #-0x10]!
;;       mov     x29, sp
;;       mov     w10, w2
;;       mov     w11, #-0xfffc
;;       adds    x10, x10, x11
;;       b.hs    #0x40
;;   18: ldr     x11, [x0, #0x68]
;;       cmp     x10, x11
;;       b.hi    #0x3c
;;   24: ldr     x13, [x0, #0x60]
;;       add     x13, x13, w2, uxtw
;;       mov     x14, #0xffff0000
;;       str     w3, [x13, x14]
;;       ldp     x29, x30, [sp], #0x10
;;       ret
;;   3c: .byte   0x1f, 0xc1, 0x00, 0x00
;;   40: .byte   0x1f, 0xc1, 0x00, 0x00
;;
;; wasm[0]::function[1]:
;;       stp     x29, x30, [sp, #-0x10]!
;;       mov     x29, sp
;;       mov     w10, w2
;;       mov     w11, #-0xfffc
;;       adds    x10, x10, x11
;;       b.hs    #0xa0
;;   78: ldr     x11, [x0, #0x68]
;;       cmp     x10, x11
;;       b.hi    #0x9c
;;   84: ldr     x13, [x0, #0x60]
;;       add     x13, x13, w2, uxtw
;;       mov     x14, #0xffff0000
;;       ldr     w0, [x13, x14]
;;       ldp     x29, x30, [sp], #0x10
;;       ret
;;   9c: .byte   0x1f, 0xc1, 0x00, 0x00
;;   a0: .byte   0x1f, 0xc1, 0x00, 0x00
