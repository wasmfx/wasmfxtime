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
;;! index_type = "i64"
;;! style = { kind = "dynamic", bound = "heap_bound" }

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i64 1)

  (func (export "do_store") (param i64 i32)
    local.get 0
    local.get 1
    i32.store offset=0xffff0000)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load offset=0xffff0000))

;; function u0:0:
;; block0:
;;   ld a3,[const(1)]
;;   add a5,a0,a3
;;   ult a3,a5,a0##ty=i64
;;   trap_if heap_oob##(a3 ne zero)
;;   ld a3,8(a2)
;;   ugt a3,a5,a3##ty=i64
;;   bne a3,zero,taken(label3),not_taken(label1)
;; block1:
;;   ld a2,0(a2)
;;   add a2,a2,a0
;;   ld a3,[const(0)]
;;   add a2,a2,a3
;;   sw a1,0(a2)
;;   j label2
;; block2:
;;   ret
;; block3:
;;   udf##trap_code=heap_oob
;;
;; function u0:1:
;; block0:
;;   ld a2,[const(1)]
;;   add a5,a0,a2
;;   ult a2,a5,a0##ty=i64
;;   trap_if heap_oob##(a2 ne zero)
;;   ld a2,8(a1)
;;   ugt a2,a5,a2##ty=i64
;;   bne a2,zero,taken(label3),not_taken(label1)
;; block1:
;;   ld a2,0(a1)
;;   add a2,a2,a0
;;   ld a3,[const(0)]
;;   add a2,a2,a3
;;   lw a0,0(a2)
;;   j label2
;; block2:
;;   ret
;; block3:
;;   udf##trap_code=heap_oob
