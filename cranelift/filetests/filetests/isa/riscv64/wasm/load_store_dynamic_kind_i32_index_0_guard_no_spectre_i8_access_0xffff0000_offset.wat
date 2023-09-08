;;! target = "riscv64"
;;!
;;! settings = ['enable_heap_access_spectre_mitigation=false']
;;!
;;! compile = true
;;!
;;! [globals.vmctx]
;;! type = "i64"
;;! vmctx = true
;;!
;;! [globals.heap_base]
;;! type = "i64"
;;! load = { base = "vmctx", offset = 0, readonly = true }
;;!
;;! [globals.heap_bound]
;;! type = "i64"
;;! load = { base = "vmctx", offset = 8, readonly = true }
;;!
;;! [[heaps]]
;;! base = "heap_base"
;;! min_size = 0x10000
;;! offset_guard_size = 0
;;! index_type = "i32"
;;! style = { kind = "dynamic", bound = "heap_bound" }

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

;; function u0:0:
;; block0:
;;   slli a3,a0,32
;;   srli a3,a3,32
;;   ld a4,[const(1)]
;;   add a4,a3,a4
;;   ult a5,a4,a3##ty=i64
;;   trap_if a5,heap_oob
;;   ld a5,8(a2)
;;   ugt a4,a4,a5##ty=i64
;;   bne a4,zero,taken(label3),not_taken(label1)
;; block1:
;;   ld a4,0(a2)
;;   add a4,a4,a3
;;   ld a5,[const(0)]
;;   add a4,a4,a5
;;   sb a1,0(a4)
;;   j label2
;; block2:
;;   ret
;; block3:
;;   udf##trap_code=heap_oob
;;
;; function u0:1:
;; block0:
;;   slli a2,a0,32
;;   srli a3,a2,32
;;   ld a2,[const(1)]
;;   add a2,a3,a2
;;   ult a4,a2,a3##ty=i64
;;   trap_if a4,heap_oob
;;   ld a4,8(a1)
;;   ugt a4,a2,a4##ty=i64
;;   bne a4,zero,taken(label3),not_taken(label1)
;; block1:
;;   ld a4,0(a1)
;;   add a4,a4,a3
;;   ld a5,[const(0)]
;;   add a4,a4,a5
;;   lbu a0,0(a4)
;;   j label2
;; block2:
;;   ret
;; block3:
;;   udf##trap_code=heap_oob
