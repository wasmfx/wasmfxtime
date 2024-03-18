;;! target = "x86_64"

(module
  (memory 1)
  (func $main (local i32 i32 i32 i32)
    (local.set 0 (i32.const 0))
    (local.set 1 (i32.const 1))
    (local.set 2 (i32.const 1))
    (local.set 3 (i32.const 0))
    (block
    (loop
        (br_if 1 (i32.gt_s (local.get 0) (i32.const 5)))
        (local.set 3 (local.get 2))
        (local.set 2 (i32.add (local.get 2) (local.get 1)))
        (local.set 1 (local.get 3))
        (local.set 0 (i32.add (local.get 0) (i32.const 1)))
        (br 0)
    )
    )
    (i32.store (i32.const 0) (local.get 2))
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
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @001f                               v2 = iconst.i32 0
;; @001f                               v3 = global_value.i64 gv3
;; @001f                               v4 = load.i64 notrap aligned v3+8
;; @0021                               v5 = iconst.i32 0
;; @0025                               v6 = iconst.i32 1
;; @0029                               v7 = iconst.i32 1
;; @002d                               v8 = iconst.i32 0
;; @0033                               jump block3(v5, v7, v6)  ; v5 = 0, v7 = 1, v6 = 1
;;
;;                                 block3(v9: i32, v13: i32, v14: i32):
;; @0037                               v10 = iconst.i32 5
;; @0039                               v11 = icmp sgt v9, v10  ; v10 = 5
;; @0039                               v12 = uextend.i32 v11
;; @003a                               brif v12, block2, block5
;;
;;                                 block5:
;; @0044                               v15 = iadd.i32 v13, v14
;; @004d                               v16 = iconst.i32 1
;; @004f                               v17 = iadd.i32 v9, v16  ; v16 = 1
;; @0052                               jump block3(v17, v15, v13)
;;
;;                                 block2:
;; @0056                               v18 = iconst.i32 0
;; @005a                               v19 = uextend.i64 v18  ; v18 = 0
;; @005a                               v20 = global_value.i64 gv4
;; @005a                               v21 = iadd v20, v19
;; @005a                               store.i32 little heap v13, v21
;; @005d                               jump block1
;;
;;                                 block1:
;; @005d                               return
;; }
