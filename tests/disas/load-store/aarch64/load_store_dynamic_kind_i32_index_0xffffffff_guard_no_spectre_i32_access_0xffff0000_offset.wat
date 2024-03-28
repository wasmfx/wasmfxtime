;;! target = "aarch64"
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
    i32.store offset=0xffff0000)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load offset=0xffff0000))

;; wasm[0]::function[0]:
;;       stp     x29, x30, [sp, #-0x10]!
;;       mov     x29, sp
;;       ldr     x9, [x0, #0x58]
;;       mov     w10, w2
;;       cmp     x10, x9
;;       b.hi    #0x30
;;   18: ldr     x11, [x0, #0x50]
;;       add     x11, x11, w2, uxtw
;;       mov     x12, #0xffff0000
;;       str     w3, [x11, x12]
;;       ldp     x29, x30, [sp], #0x10
;;       ret
;;   30: .byte   0x1f, 0xc1, 0x00, 0x00
;;
;; wasm[0]::function[1]:
;;       stp     x29, x30, [sp, #-0x10]!
;;       mov     x29, sp
;;       ldr     x9, [x0, #0x58]
;;       mov     w10, w2
;;       cmp     x10, x9
;;       b.hi    #0x70
;;   58: ldr     x11, [x0, #0x50]
;;       add     x11, x11, w2, uxtw
;;       mov     x12, #0xffff0000
;;       ldr     w0, [x11, x12]
;;       ldp     x29, x30, [sp], #0x10
;;       ret
;;   70: .byte   0x1f, 0xc1, 0x00, 0x00
