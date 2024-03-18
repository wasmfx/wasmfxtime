;;! target = "x86_64"

(module
  (func $main
    (v128.const i32x4 1 2 3 4)
    (v128.const i32x4 1 2 3 4)
    (call $add)
    drop
  )
  (func $add (param $a v128) (param $b v128) (result v128)
    (local.get $a)
    (local.get $b)
    (i32x4.add)
  )
  (start $main)
)

;; function u0:0(i64 vmctx, i64) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i64, i8x16, i8x16) -> i8x16 fast
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     fn0 = colocated u0:1 sig0
;;     const0 = 0x00000004000000030000000200000001
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @0020                               v2 = global_value.i64 gv3
;; @0020                               v3 = load.i64 notrap aligned v2+8
;; @0021                               v4 = vconst.i8x16 const0
;; @0033                               v5 = vconst.i8x16 const0
;; @0045                               v6 = call fn0(v0, v0, v4, v5)  ; v4 = const0, v5 = const0
;; @0048                               jump block1
;;
;;                                 block1:
;; @0048                               return
;; }
;;
;; function u0:1(i64 vmctx, i64, i8x16, i8x16) -> i8x16 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i8x16, v3: i8x16):
;; @004a                               v5 = global_value.i64 gv3
;; @004a                               v6 = load.i64 notrap aligned v5+8
;; @004f                               v7 = bitcast.i32x4 little v2
;; @004f                               v8 = bitcast.i32x4 little v3
;; @004f                               v9 = iadd v7, v8
;; @0052                               v10 = bitcast.i8x16 little v9
;; @0052                               jump block1(v10)
;;
;;                                 block1(v4: i8x16):
;; @0052                               return v4
;; }
