;;! target = "x86_64"

(module
  (memory 1)
  (func $main (local i32)
    (i32.store (i32.const 0) (i32.const 0x0))
    (if (i32.load (i32.const 0))
        (then (i32.store (i32.const 0) (i32.const 0xa)))
        (else (i32.store (i32.const 0) (i32.const 0xb))))
  )
  (start $main)
  (data (i32.const 0) "0000")
)

;; function u0:0(i64 vmctx, i64) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned readonly checked gv3+80
;;     sig0 = (i64 vmctx, i32 uext, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig3 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @001f                               v2 = iconst.i32 0
;; @001f                               v3 = global_value.i64 gv3
;; @001f                               v4 = load.i64 notrap aligned v3+8
;; @0021                               v5 = iconst.i32 0
;; @0023                               v6 = iconst.i32 0
;; @0025                               v7 = uextend.i64 v5  ; v5 = 0
;; @0025                               v8 = global_value.i64 gv4
;; @0025                               v9 = iadd v8, v7
;; @0025                               store little heap v6, v9  ; v6 = 0
;; @0028                               v10 = iconst.i32 0
;; @002a                               v11 = uextend.i64 v10  ; v10 = 0
;; @002a                               v12 = global_value.i64 gv4
;; @002a                               v13 = iadd v12, v11
;; @002a                               v14 = load.i32 little heap v13
;; @002d                               brif v14, block2, block4
;;
;;                                 block2:
;; @002f                               v15 = iconst.i32 0
;; @0031                               v16 = iconst.i32 10
;; @0033                               v17 = uextend.i64 v15  ; v15 = 0
;; @0033                               v18 = global_value.i64 gv4
;; @0033                               v19 = iadd v18, v17
;; @0033                               store little heap v16, v19  ; v16 = 10
;; @0036                               jump block3
;;
;;                                 block4:
;; @0037                               v20 = iconst.i32 0
;; @0039                               v21 = iconst.i32 11
;; @003b                               v22 = uextend.i64 v20  ; v20 = 0
;; @003b                               v23 = global_value.i64 gv4
;; @003b                               v24 = iadd v23, v22
;; @003b                               store little heap v21, v24  ; v21 = 11
;; @003e                               jump block3
;;
;;                                 block3:
;; @003f                               jump block1
;;
;;                                 block1:
;; @003f                               return
;; }
