;;! target = "x86_64"

(module
  (func $untyped-select (result i32)
  	i32.const 42
  	i32.const 24
  	i32.const 1
  	select)

  (func $typed-select-1 (result externref)
  	ref.null extern
  	ref.null extern
  	i32.const 1
  	select (result externref))

  (func $typed-select-2 (param externref) (result externref)
    ref.null extern
    local.get 0
    i32.const 1
    select (result externref))
)

;; function u0:0(i64 vmctx, i64) -> i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @0022                               v3 = global_value.i64 gv3
;; @0022                               v4 = load.i64 notrap aligned v3+8
;; @0023                               v5 = iconst.i32 42
;; @0025                               v6 = iconst.i32 24
;; @0027                               v7 = iconst.i32 1
;; @0029                               v8 = select v7, v5, v6  ; v7 = 1, v5 = 42, v6 = 24
;; @002a                               jump block1(v8)
;;
;;                                 block1(v2: i32):
;; @002a                               return v2
;; }
;;
;; function u0:1(i64 vmctx, i64) -> r64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @002c                               v3 = global_value.i64 gv3
;; @002c                               v4 = load.i64 notrap aligned v3+8
;; @002d                               v5 = null.r64 
;; @002f                               v6 = null.r64 
;; @0031                               v7 = iconst.i32 1
;; @0033                               v8 = select v7, v5, v6  ; v7 = 1
;; @0036                               jump block1(v8)
;;
;;                                 block1(v2: r64):
;; @0036                               return v2
;; }
;;
;; function u0:2(i64 vmctx, i64, r64) -> r64 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: r64):
;; @0038                               v4 = global_value.i64 gv3
;; @0038                               v5 = load.i64 notrap aligned v4+8
;; @0039                               v6 = null.r64 
;; @003d                               v7 = iconst.i32 1
;; @003f                               v8 = select v7, v6, v2  ; v7 = 1
;; @0042                               jump block1(v8)
;;
;;                                 block1(v3: r64):
;; @0042                               return v3
;; }
