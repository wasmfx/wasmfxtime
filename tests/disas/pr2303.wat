;;! target = "x86_64"

(module
    (memory (export "mem") 1 1)
    (func (export "runif") (param $cond i32)
      i32.const 48
      (v128.load (i32.const 0))
      (v128.load (i32.const 16))
      (if (param v128) (param v128) (result v128 v128)
          (local.get $cond)
          (then i64x2.add
                (v128.load (i32.const 32)))
          (else i32x4.sub
                (v128.load (i32.const 0))))
      i16x8.mul
      v128.store)
)

;; function u0:0(i64 vmctx, i64, i32) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i32):
;; @0035                               v3 = global_value.i64 gv3
;; @0035                               v4 = load.i64 notrap aligned v3+8
;; @0036                               v5 = iconst.i32 48
;; @0038                               v6 = iconst.i32 0
;; @003a                               v7 = uextend.i64 v6  ; v6 = 0
;; @003a                               v8 = global_value.i64 gv4
;; @003a                               v9 = iadd v8, v7
;; @003a                               v10 = load.i8x16 little heap v9
;; @003e                               v11 = iconst.i32 16
;; @0040                               v12 = uextend.i64 v11  ; v11 = 16
;; @0040                               v13 = global_value.i64 gv4
;; @0040                               v14 = iadd v13, v12
;; @0040                               v15 = load.i8x16 little heap v14
;; @0046                               brif v2, block2, block4(v10, v15)
;;
;;                                 block2:
;; @0048                               v18 = bitcast.i64x2 little v10
;; @0048                               v19 = bitcast.i64x2 little v15
;; @0048                               v20 = iadd v18, v19
;; @004b                               v21 = iconst.i32 32
;; @004d                               v22 = uextend.i64 v21  ; v21 = 32
;; @004d                               v23 = global_value.i64 gv4
;; @004d                               v24 = iadd v23, v22
;; @004d                               v25 = load.i8x16 little heap v24
;; @0051                               v28 = bitcast.i8x16 little v20
;; @0051                               jump block3(v28, v25)
;;
;;                                 block4(v26: i8x16, v27: i8x16):
;; @0052                               v29 = bitcast.i32x4 little v10
;; @0052                               v30 = bitcast.i32x4 little v15
;; @0052                               v31 = isub v29, v30
;; @0055                               v32 = iconst.i32 0
;; @0057                               v33 = uextend.i64 v32  ; v32 = 0
;; @0057                               v34 = global_value.i64 gv4
;; @0057                               v35 = iadd v34, v33
;; @0057                               v36 = load.i8x16 little heap v35
;; @005b                               v37 = bitcast.i8x16 little v31
;; @005b                               jump block3(v37, v36)
;;
;;                                 block3(v16: i8x16, v17: i8x16):
;; @005c                               v38 = bitcast.i16x8 little v16
;; @005c                               v39 = bitcast.i16x8 little v17
;; @005c                               v40 = imul v38, v39
;; @005f                               v41 = uextend.i64 v5  ; v5 = 48
;; @005f                               v42 = global_value.i64 gv4
;; @005f                               v43 = iadd v42, v41
;; @005f                               store little heap v40, v43
;; @0063                               jump block1
;;
;;                                 block1:
;; @0063                               return
;; }
