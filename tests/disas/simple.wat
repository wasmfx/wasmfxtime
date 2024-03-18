;;! target = "x86_64"

(module
    (func $small1 (param i32) (result i32)
        (i32.add (local.get 0) (i32.const 1))
    )

    (func $small2 (param i32) (result i32)
        (return (i32.add (local.get 0) (i32.const 1)))
    )

    (func $infloop (result i32)
        (local i32)
        (loop (result i32)
            (i32.add (local.get 0) (i32.const 1))
            (local.set 0)
            (br 0)
        )
    )
)
;; function u0:0(i64 vmctx, i64, i32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;; @001e                               v4 = global_value.i64 gv3
;; @001e                               v5 = load.i64 notrap aligned v4+8
;; @0021                               v6 = iconst.i32 1
;; @0023                               v7 = iadd v2, v6  ; v6 = 1
;; @0024                               jump block1(v7)
;;
;;                                 block1(v3: i32):
;; @0024                               return v3
;; }
;;
;; function u0:1(i64 vmctx, i64, i32) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;; @0026                               v4 = global_value.i64 gv3
;; @0026                               v5 = load.i64 notrap aligned v4+8
;; @0029                               v6 = iconst.i32 1
;; @002b                               v7 = iadd v2, v6  ; v6 = 1
;; @002c                               return v7
;; }
;;
;; function u0:2(i64 vmctx, i64) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @0030                               v3 = iconst.i32 0
;; @0030                               v4 = global_value.i64 gv3
;; @0030                               v5 = load.i64 notrap aligned v4+8
;; @0032                               jump block2(v3)  ; v3 = 0
;;
;;                                 block2(v7: i32):
;; @0036                               v8 = iconst.i32 1
;; @0038                               v9 = iadd v7, v8  ; v8 = 1
;; @003b                               jump block2(v9)
;; }
