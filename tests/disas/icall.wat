;;! target = "x86_64"

(module
  (type $ft (func (param f32) (result i32)))
  (func $foo (export "foo") (param i32 f32) (result i32)
    (call_indirect (type $ft) (local.get 1) (local.get 0))
  )
  (table (;0;) 23 23 funcref)
)

;; function u0:0(i64 vmctx, i64, i32, f32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+72
;;     sig0 = (i64 vmctx, i64, f32) -> i32 fast
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i64 system_v
;;     sig2 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig3 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32, v3: f32):
;; @002e                               v5 = global_value.i64 gv3
;; @002e                               v6 = load.i64 notrap aligned v5+8
;; @0033                               v7 = iconst.i32 23
;; @0033                               v8 = icmp uge v2, v7  ; v7 = 23
;; @0033                               trapnz v8, table_oob
;; @0033                               v9 = uextend.i64 v2
;; @0033                               v10 = global_value.i64 gv4
;; @0033                               v11 = ishl_imm v9, 3
;; @0033                               v12 = iadd v10, v11
;; @0033                               v13 = icmp uge v2, v7  ; v7 = 23
;; @0033                               v14 = select_spectre_guard v13, v10, v12
;; @0033                               v15 = load.i64 notrap aligned table v14
;; @0033                               v16 = band_imm v15, -2
;; @0033                               brif v15, block3(v16), block2
;;
;;                                 block2 cold:
;; @0033                               v18 = iconst.i32 0
;; @0033                               v19 = global_value.i64 gv3
;; @0033                               v20 = load.i64 notrap aligned readonly v19+56
;; @0033                               v21 = load.i64 notrap aligned readonly v20+72
;; @0033                               v22 = call_indirect sig1, v21(v19, v18, v2)  ; v18 = 0
;; @0033                               jump block3(v22)
;;
;;                                 block3(v17: i64):
;; @0033                               trapz v17, icall_null
;; @0033                               v23 = global_value.i64 gv3
;; @0033                               v24 = load.i64 notrap aligned readonly v23+64
;; @0033                               v25 = load.i32 notrap aligned readonly v24
;; @0033                               v26 = load.i32 notrap aligned readonly v17+24
;; @0033                               v27 = icmp eq v26, v25
;; @0033                               trapz v27, bad_sig
;; @0033                               v28 = load.i64 notrap aligned readonly v17+16
;; @0033                               v29 = load.i64 notrap aligned readonly v17+32
;; @0033                               v30 = call_indirect sig0, v28(v29, v0, v3)
;; @0036                               jump block1(v30)
;;
;;                                 block1(v4: i32):
;; @0036                               return v4
;; }
