;;! target = "riscv64"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -O static-memory-maximum-size=0 -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i32 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store8 offset=0x1000)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load8_u offset=0x1000))

;; function u0:0:
;; block0:
;;   ld a1,88(a0)
;;   ld a0,80(a0)
;;   slli a5,a2,32
;;   srli a2,a5,32
;;   lui a5,1
;;   addi a4,a5,1
;;   sub a1,a1,a4
;;   sltu a1,a1,a2
;;   add a0,a0,a2
;;   lui a2,1
;;   add a0,a0,a2
;;   sub a4,zero,a1
;;   not a1,a4
;;   and a2,a0,a1
;;   sb a3,0(a2)
;;   j label1
;; block1:
;;   ret
;;
;; function u0:1:
;; block0:
;;   ld a1,88(a0)
;;   ld a0,80(a0)
;;   slli a5,a2,32
;;   srli a2,a5,32
;;   lui a5,1
;;   addi a3,a5,1
;;   sub a1,a1,a3
;;   sltu a1,a1,a2
;;   add a0,a0,a2
;;   lui a2,1
;;   add a0,a0,a2
;;   sub a4,zero,a1
;;   not a1,a4
;;   and a2,a0,a1
;;   lbu a0,0(a2)
;;   j label1
;; block1:
;;   ret
