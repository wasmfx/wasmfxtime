;;! target = "x86_64"

(module
  (type (;0;) (func (result v128 v128 v128)))

  (func $main1 (type 0) (result v128 v128 v128)
    call $main1
    i8x16.add
    call $main1
    i8x16.ge_u
    i16x8.ne
    i32.const 13
    br_if 0 (;@0;)
    i32.const 43
    br_if 0 (;@0;)
    i32.const 13
    br_if 0 (;@0;)
    i32.const 87
    select
    unreachable
    i32.const 0
    br_if 0 (;@0;)
    i32.const 13
    br_if 0 (;@0;)
    i32.const 43
    br_if 0 (;@0;)
  )
  (export "main1" (func $main1))

  (func $main2 (type 0) (result v128 v128 v128)
    call $main2
    i8x16.add
    call $main2
    i8x16.ge_u
    i16x8.ne
    i32.const 13
    br_if 0 (;@0;)
    i32.const 43
    br_if 0 (;@0;)
    i32.const 13
    br_if 0 (;@0;)
    i32.const 87
    select (result v128)
    unreachable
    i32.const 0
    br_if 0 (;@0;)
    i32.const 13
    br_if 0 (;@0;)
    i32.const 43
    br_if 0 (;@0;)
  )
  (export "main2" (func $main2))
)

;; function u0:0(i64 vmctx, i64) -> i8x16, i8x16, i8x16 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i64) -> i8x16, i8x16, i8x16 fast
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     fn0 = colocated u0:0 sig0
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @002d                               v5 = global_value.i64 gv3
;; @002d                               v6 = load.i64 notrap aligned v5+8
;; @002e                               v7, v8, v9 = call fn0(v0, v0)
;; @0030                               v10 = iadd v8, v9
;; @0032                               v11, v12, v13 = call fn0(v0, v0)
;; @0034                               v14 = icmp uge v12, v13
;; @0036                               v15 = bitcast.i16x8 little v11
;; @0036                               v16 = bitcast.i16x8 little v14
;; @0036                               v17 = icmp ne v15, v16
;; @0038                               v18 = iconst.i32 13
;; @003a                               v19 = bitcast.i8x16 little v17
;; @003a                               brif v18, block1(v7, v10, v19), block2  ; v18 = 13
;;
;;                                 block2:
;; @003c                               v20 = iconst.i32 43
;; @003e                               v21 = bitcast.i8x16 little v17
;; @003e                               brif v20, block1(v7, v10, v21), block3  ; v20 = 43
;;
;;                                 block3:
;; @0040                               v22 = iconst.i32 13
;; @0042                               v23 = bitcast.i8x16 little v17
;; @0042                               brif v22, block1(v7, v10, v23), block4  ; v22 = 13
;;
;;                                 block4:
;; @0044                               v24 = iconst.i32 87
;; @0047                               v25 = bitcast.i8x16 little v17
;; @0047                               v26 = select.i8x16 v24, v10, v25  ; v24 = 87
;; @0048                               trap unreachable
;;
;;                                 block1(v2: i8x16, v3: i8x16, v4: i8x16):
;; @0055                               return v2, v3, v4
;; }
;;
;; function u0:1(i64 vmctx, i64) -> i8x16, i8x16, i8x16 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     sig0 = (i64 vmctx, i64) -> i8x16, i8x16, i8x16 fast
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     fn0 = colocated u0:1 sig0
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64):
;; @0057                               v5 = global_value.i64 gv3
;; @0057                               v6 = load.i64 notrap aligned v5+8
;; @0058                               v7, v8, v9 = call fn0(v0, v0)
;; @005a                               v10 = iadd v8, v9
;; @005c                               v11, v12, v13 = call fn0(v0, v0)
;; @005e                               v14 = icmp uge v12, v13
;; @0060                               v15 = bitcast.i16x8 little v11
;; @0060                               v16 = bitcast.i16x8 little v14
;; @0060                               v17 = icmp ne v15, v16
;; @0062                               v18 = iconst.i32 13
;; @0064                               v19 = bitcast.i8x16 little v17
;; @0064                               brif v18, block1(v7, v10, v19), block2  ; v18 = 13
;;
;;                                 block2:
;; @0066                               v20 = iconst.i32 43
;; @0068                               v21 = bitcast.i8x16 little v17
;; @0068                               brif v20, block1(v7, v10, v21), block3  ; v20 = 43
;;
;;                                 block3:
;; @006a                               v22 = iconst.i32 13
;; @006c                               v23 = bitcast.i8x16 little v17
;; @006c                               brif v22, block1(v7, v10, v23), block4  ; v22 = 13
;;
;;                                 block4:
;; @006e                               v24 = iconst.i32 87
;; @0071                               v25 = bitcast.i8x16 little v17
;; @0071                               v26 = select.i8x16 v24, v10, v25  ; v24 = 87
;; @0074                               trap unreachable
;;
;;                                 block1(v2: i8x16, v3: i8x16, v4: i8x16):
;; @0081                               return v2, v3, v4
;; }
