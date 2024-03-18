;;! target = "x86_64"

(module
  (func (param v128)
    (v128.store (i32.const 0) (i8x16.eq (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i16x8.eq (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i32x4.eq (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i64x2.eq (local.get 0) (local.get 0))))

  (func (param v128)
    (v128.store (i32.const 0) (i8x16.ne (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i16x8.ne (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i32x4.ne (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i64x2.ne (local.get 0) (local.get 0))))

  (func (param v128)
    (v128.store (i32.const 0) (i8x16.lt_s (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i16x8.lt_s (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i32x4.lt_s (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i64x2.lt_s (local.get 0) (local.get 0))))

  (func (param v128)
    (v128.store (i32.const 0) (i8x16.lt_u (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i16x8.lt_u (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i32x4.lt_u (local.get 0) (local.get 0))))

  (func (param v128)
    (v128.store (i32.const 0) (i8x16.gt_s (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i16x8.gt_s (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i32x4.gt_s (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i64x2.gt_s (local.get 0) (local.get 0))))

  (func (param v128)
    (v128.store (i32.const 0) (i8x16.gt_u (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i16x8.gt_u (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (i32x4.gt_u (local.get 0) (local.get 0))))

  (func (param v128)
    (v128.store (i32.const 0) (f32x4.eq (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (f64x2.eq (local.get 0) (local.get 0))))

  (func (param v128)
    (v128.store (i32.const 0) (f32x4.ne (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (f64x2.ne (local.get 0) (local.get 0))))

  (func (param v128)
    (v128.store (i32.const 0) (f32x4.lt (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (f64x2.lt (local.get 0) (local.get 0))))

  (func (param v128)
    (v128.store (i32.const 0) (f32x4.le (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (f64x2.le (local.get 0) (local.get 0))))

  (func (param v128)
    (v128.store (i32.const 0) (f32x4.gt (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (f64x2.gt (local.get 0) (local.get 0))))

  (func (param v128)
    (v128.store (i32.const 0) (f32x4.ge (local.get 0) (local.get 0))))
  (func (param v128)
    (v128.store (i32.const 0) (f64x2.ge (local.get 0) (local.get 0))))

  (memory 0)
)

;; function u0:0(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @003e                               v3 = global_value.i64 gv3
;; @003e                               v4 = load.i64 notrap aligned v3+8
;; @003f                               v5 = iconst.i32 0
;; @0045                               v6 = icmp eq v2, v2
;; @0047                               v7 = uextend.i64 v5  ; v5 = 0
;; @0047                               v8 = global_value.i64 gv4
;; @0047                               v9 = iadd v8, v7
;; @0047                               store little heap v6, v9
;; @004b                               jump block1
;;
;;                                 block1:
;; @004b                               return
;; }
;;
;; function u0:1(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @004d                               v3 = global_value.i64 gv3
;; @004d                               v4 = load.i64 notrap aligned v3+8
;; @004e                               v5 = iconst.i32 0
;; @0054                               v6 = bitcast.i16x8 little v2
;; @0054                               v7 = bitcast.i16x8 little v2
;; @0054                               v8 = icmp eq v6, v7
;; @0056                               v9 = uextend.i64 v5  ; v5 = 0
;; @0056                               v10 = global_value.i64 gv4
;; @0056                               v11 = iadd v10, v9
;; @0056                               store little heap v8, v11
;; @005a                               jump block1
;;
;;                                 block1:
;; @005a                               return
;; }
;;
;; function u0:2(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @005c                               v3 = global_value.i64 gv3
;; @005c                               v4 = load.i64 notrap aligned v3+8
;; @005d                               v5 = iconst.i32 0
;; @0063                               v6 = bitcast.i32x4 little v2
;; @0063                               v7 = bitcast.i32x4 little v2
;; @0063                               v8 = icmp eq v6, v7
;; @0065                               v9 = uextend.i64 v5  ; v5 = 0
;; @0065                               v10 = global_value.i64 gv4
;; @0065                               v11 = iadd v10, v9
;; @0065                               store little heap v8, v11
;; @0069                               jump block1
;;
;;                                 block1:
;; @0069                               return
;; }
;;
;; function u0:3(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @006b                               v3 = global_value.i64 gv3
;; @006b                               v4 = load.i64 notrap aligned v3+8
;; @006c                               v5 = iconst.i32 0
;; @0072                               v6 = bitcast.i64x2 little v2
;; @0072                               v7 = bitcast.i64x2 little v2
;; @0072                               v8 = icmp eq v6, v7
;; @0075                               v9 = uextend.i64 v5  ; v5 = 0
;; @0075                               v10 = global_value.i64 gv4
;; @0075                               v11 = iadd v10, v9
;; @0075                               store little heap v8, v11
;; @0079                               jump block1
;;
;;                                 block1:
;; @0079                               return
;; }
;;
;; function u0:4(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @007b                               v3 = global_value.i64 gv3
;; @007b                               v4 = load.i64 notrap aligned v3+8
;; @007c                               v5 = iconst.i32 0
;; @0082                               v6 = icmp ne v2, v2
;; @0084                               v7 = uextend.i64 v5  ; v5 = 0
;; @0084                               v8 = global_value.i64 gv4
;; @0084                               v9 = iadd v8, v7
;; @0084                               store little heap v6, v9
;; @0088                               jump block1
;;
;;                                 block1:
;; @0088                               return
;; }
;;
;; function u0:5(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @008a                               v3 = global_value.i64 gv3
;; @008a                               v4 = load.i64 notrap aligned v3+8
;; @008b                               v5 = iconst.i32 0
;; @0091                               v6 = bitcast.i16x8 little v2
;; @0091                               v7 = bitcast.i16x8 little v2
;; @0091                               v8 = icmp ne v6, v7
;; @0093                               v9 = uextend.i64 v5  ; v5 = 0
;; @0093                               v10 = global_value.i64 gv4
;; @0093                               v11 = iadd v10, v9
;; @0093                               store little heap v8, v11
;; @0097                               jump block1
;;
;;                                 block1:
;; @0097                               return
;; }
;;
;; function u0:6(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @0099                               v3 = global_value.i64 gv3
;; @0099                               v4 = load.i64 notrap aligned v3+8
;; @009a                               v5 = iconst.i32 0
;; @00a0                               v6 = bitcast.i32x4 little v2
;; @00a0                               v7 = bitcast.i32x4 little v2
;; @00a0                               v8 = icmp ne v6, v7
;; @00a2                               v9 = uextend.i64 v5  ; v5 = 0
;; @00a2                               v10 = global_value.i64 gv4
;; @00a2                               v11 = iadd v10, v9
;; @00a2                               store little heap v8, v11
;; @00a6                               jump block1
;;
;;                                 block1:
;; @00a6                               return
;; }
;;
;; function u0:7(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @00a8                               v3 = global_value.i64 gv3
;; @00a8                               v4 = load.i64 notrap aligned v3+8
;; @00a9                               v5 = iconst.i32 0
;; @00af                               v6 = bitcast.i64x2 little v2
;; @00af                               v7 = bitcast.i64x2 little v2
;; @00af                               v8 = icmp ne v6, v7
;; @00b2                               v9 = uextend.i64 v5  ; v5 = 0
;; @00b2                               v10 = global_value.i64 gv4
;; @00b2                               v11 = iadd v10, v9
;; @00b2                               store little heap v8, v11
;; @00b6                               jump block1
;;
;;                                 block1:
;; @00b6                               return
;; }
;;
;; function u0:8(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @00b8                               v3 = global_value.i64 gv3
;; @00b8                               v4 = load.i64 notrap aligned v3+8
;; @00b9                               v5 = iconst.i32 0
;; @00bf                               v6 = icmp slt v2, v2
;; @00c1                               v7 = uextend.i64 v5  ; v5 = 0
;; @00c1                               v8 = global_value.i64 gv4
;; @00c1                               v9 = iadd v8, v7
;; @00c1                               store little heap v6, v9
;; @00c5                               jump block1
;;
;;                                 block1:
;; @00c5                               return
;; }
;;
;; function u0:9(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @00c7                               v3 = global_value.i64 gv3
;; @00c7                               v4 = load.i64 notrap aligned v3+8
;; @00c8                               v5 = iconst.i32 0
;; @00ce                               v6 = bitcast.i16x8 little v2
;; @00ce                               v7 = bitcast.i16x8 little v2
;; @00ce                               v8 = icmp slt v6, v7
;; @00d0                               v9 = uextend.i64 v5  ; v5 = 0
;; @00d0                               v10 = global_value.i64 gv4
;; @00d0                               v11 = iadd v10, v9
;; @00d0                               store little heap v8, v11
;; @00d4                               jump block1
;;
;;                                 block1:
;; @00d4                               return
;; }
;;
;; function u0:10(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @00d6                               v3 = global_value.i64 gv3
;; @00d6                               v4 = load.i64 notrap aligned v3+8
;; @00d7                               v5 = iconst.i32 0
;; @00dd                               v6 = bitcast.i32x4 little v2
;; @00dd                               v7 = bitcast.i32x4 little v2
;; @00dd                               v8 = icmp slt v6, v7
;; @00df                               v9 = uextend.i64 v5  ; v5 = 0
;; @00df                               v10 = global_value.i64 gv4
;; @00df                               v11 = iadd v10, v9
;; @00df                               store little heap v8, v11
;; @00e3                               jump block1
;;
;;                                 block1:
;; @00e3                               return
;; }
;;
;; function u0:11(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @00e5                               v3 = global_value.i64 gv3
;; @00e5                               v4 = load.i64 notrap aligned v3+8
;; @00e6                               v5 = iconst.i32 0
;; @00ec                               v6 = bitcast.i64x2 little v2
;; @00ec                               v7 = bitcast.i64x2 little v2
;; @00ec                               v8 = icmp slt v6, v7
;; @00ef                               v9 = uextend.i64 v5  ; v5 = 0
;; @00ef                               v10 = global_value.i64 gv4
;; @00ef                               v11 = iadd v10, v9
;; @00ef                               store little heap v8, v11
;; @00f3                               jump block1
;;
;;                                 block1:
;; @00f3                               return
;; }
;;
;; function u0:12(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @00f5                               v3 = global_value.i64 gv3
;; @00f5                               v4 = load.i64 notrap aligned v3+8
;; @00f6                               v5 = iconst.i32 0
;; @00fc                               v6 = icmp ult v2, v2
;; @00fe                               v7 = uextend.i64 v5  ; v5 = 0
;; @00fe                               v8 = global_value.i64 gv4
;; @00fe                               v9 = iadd v8, v7
;; @00fe                               store little heap v6, v9
;; @0102                               jump block1
;;
;;                                 block1:
;; @0102                               return
;; }
;;
;; function u0:13(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @0104                               v3 = global_value.i64 gv3
;; @0104                               v4 = load.i64 notrap aligned v3+8
;; @0105                               v5 = iconst.i32 0
;; @010b                               v6 = bitcast.i16x8 little v2
;; @010b                               v7 = bitcast.i16x8 little v2
;; @010b                               v8 = icmp ult v6, v7
;; @010d                               v9 = uextend.i64 v5  ; v5 = 0
;; @010d                               v10 = global_value.i64 gv4
;; @010d                               v11 = iadd v10, v9
;; @010d                               store little heap v8, v11
;; @0111                               jump block1
;;
;;                                 block1:
;; @0111                               return
;; }
;;
;; function u0:14(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @0113                               v3 = global_value.i64 gv3
;; @0113                               v4 = load.i64 notrap aligned v3+8
;; @0114                               v5 = iconst.i32 0
;; @011a                               v6 = bitcast.i32x4 little v2
;; @011a                               v7 = bitcast.i32x4 little v2
;; @011a                               v8 = icmp ult v6, v7
;; @011c                               v9 = uextend.i64 v5  ; v5 = 0
;; @011c                               v10 = global_value.i64 gv4
;; @011c                               v11 = iadd v10, v9
;; @011c                               store little heap v8, v11
;; @0120                               jump block1
;;
;;                                 block1:
;; @0120                               return
;; }
;;
;; function u0:15(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @0122                               v3 = global_value.i64 gv3
;; @0122                               v4 = load.i64 notrap aligned v3+8
;; @0123                               v5 = iconst.i32 0
;; @0129                               v6 = icmp sgt v2, v2
;; @012b                               v7 = uextend.i64 v5  ; v5 = 0
;; @012b                               v8 = global_value.i64 gv4
;; @012b                               v9 = iadd v8, v7
;; @012b                               store little heap v6, v9
;; @012f                               jump block1
;;
;;                                 block1:
;; @012f                               return
;; }
;;
;; function u0:16(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @0131                               v3 = global_value.i64 gv3
;; @0131                               v4 = load.i64 notrap aligned v3+8
;; @0132                               v5 = iconst.i32 0
;; @0138                               v6 = bitcast.i16x8 little v2
;; @0138                               v7 = bitcast.i16x8 little v2
;; @0138                               v8 = icmp sgt v6, v7
;; @013a                               v9 = uextend.i64 v5  ; v5 = 0
;; @013a                               v10 = global_value.i64 gv4
;; @013a                               v11 = iadd v10, v9
;; @013a                               store little heap v8, v11
;; @013e                               jump block1
;;
;;                                 block1:
;; @013e                               return
;; }
;;
;; function u0:17(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @0140                               v3 = global_value.i64 gv3
;; @0140                               v4 = load.i64 notrap aligned v3+8
;; @0141                               v5 = iconst.i32 0
;; @0147                               v6 = bitcast.i32x4 little v2
;; @0147                               v7 = bitcast.i32x4 little v2
;; @0147                               v8 = icmp sgt v6, v7
;; @0149                               v9 = uextend.i64 v5  ; v5 = 0
;; @0149                               v10 = global_value.i64 gv4
;; @0149                               v11 = iadd v10, v9
;; @0149                               store little heap v8, v11
;; @014d                               jump block1
;;
;;                                 block1:
;; @014d                               return
;; }
;;
;; function u0:18(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @014f                               v3 = global_value.i64 gv3
;; @014f                               v4 = load.i64 notrap aligned v3+8
;; @0150                               v5 = iconst.i32 0
;; @0156                               v6 = bitcast.i64x2 little v2
;; @0156                               v7 = bitcast.i64x2 little v2
;; @0156                               v8 = icmp sgt v6, v7
;; @0159                               v9 = uextend.i64 v5  ; v5 = 0
;; @0159                               v10 = global_value.i64 gv4
;; @0159                               v11 = iadd v10, v9
;; @0159                               store little heap v8, v11
;; @015d                               jump block1
;;
;;                                 block1:
;; @015d                               return
;; }
;;
;; function u0:19(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @015f                               v3 = global_value.i64 gv3
;; @015f                               v4 = load.i64 notrap aligned v3+8
;; @0160                               v5 = iconst.i32 0
;; @0166                               v6 = icmp ugt v2, v2
;; @0168                               v7 = uextend.i64 v5  ; v5 = 0
;; @0168                               v8 = global_value.i64 gv4
;; @0168                               v9 = iadd v8, v7
;; @0168                               store little heap v6, v9
;; @016c                               jump block1
;;
;;                                 block1:
;; @016c                               return
;; }
;;
;; function u0:20(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @016e                               v3 = global_value.i64 gv3
;; @016e                               v4 = load.i64 notrap aligned v3+8
;; @016f                               v5 = iconst.i32 0
;; @0175                               v6 = bitcast.i16x8 little v2
;; @0175                               v7 = bitcast.i16x8 little v2
;; @0175                               v8 = icmp ugt v6, v7
;; @0177                               v9 = uextend.i64 v5  ; v5 = 0
;; @0177                               v10 = global_value.i64 gv4
;; @0177                               v11 = iadd v10, v9
;; @0177                               store little heap v8, v11
;; @017b                               jump block1
;;
;;                                 block1:
;; @017b                               return
;; }
;;
;; function u0:21(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @017d                               v3 = global_value.i64 gv3
;; @017d                               v4 = load.i64 notrap aligned v3+8
;; @017e                               v5 = iconst.i32 0
;; @0184                               v6 = bitcast.i32x4 little v2
;; @0184                               v7 = bitcast.i32x4 little v2
;; @0184                               v8 = icmp ugt v6, v7
;; @0186                               v9 = uextend.i64 v5  ; v5 = 0
;; @0186                               v10 = global_value.i64 gv4
;; @0186                               v11 = iadd v10, v9
;; @0186                               store little heap v8, v11
;; @018a                               jump block1
;;
;;                                 block1:
;; @018a                               return
;; }
;;
;; function u0:22(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @018c                               v3 = global_value.i64 gv3
;; @018c                               v4 = load.i64 notrap aligned v3+8
;; @018d                               v5 = iconst.i32 0
;; @0193                               v6 = bitcast.f32x4 little v2
;; @0193                               v7 = bitcast.f32x4 little v2
;; @0193                               v8 = fcmp eq v6, v7
;; @0195                               v9 = uextend.i64 v5  ; v5 = 0
;; @0195                               v10 = global_value.i64 gv4
;; @0195                               v11 = iadd v10, v9
;; @0195                               store little heap v8, v11
;; @0199                               jump block1
;;
;;                                 block1:
;; @0199                               return
;; }
;;
;; function u0:23(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @019b                               v3 = global_value.i64 gv3
;; @019b                               v4 = load.i64 notrap aligned v3+8
;; @019c                               v5 = iconst.i32 0
;; @01a2                               v6 = bitcast.f64x2 little v2
;; @01a2                               v7 = bitcast.f64x2 little v2
;; @01a2                               v8 = fcmp eq v6, v7
;; @01a4                               v9 = uextend.i64 v5  ; v5 = 0
;; @01a4                               v10 = global_value.i64 gv4
;; @01a4                               v11 = iadd v10, v9
;; @01a4                               store little heap v8, v11
;; @01a8                               jump block1
;;
;;                                 block1:
;; @01a8                               return
;; }
;;
;; function u0:24(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @01aa                               v3 = global_value.i64 gv3
;; @01aa                               v4 = load.i64 notrap aligned v3+8
;; @01ab                               v5 = iconst.i32 0
;; @01b1                               v6 = bitcast.f32x4 little v2
;; @01b1                               v7 = bitcast.f32x4 little v2
;; @01b1                               v8 = fcmp ne v6, v7
;; @01b3                               v9 = uextend.i64 v5  ; v5 = 0
;; @01b3                               v10 = global_value.i64 gv4
;; @01b3                               v11 = iadd v10, v9
;; @01b3                               store little heap v8, v11
;; @01b7                               jump block1
;;
;;                                 block1:
;; @01b7                               return
;; }
;;
;; function u0:25(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @01b9                               v3 = global_value.i64 gv3
;; @01b9                               v4 = load.i64 notrap aligned v3+8
;; @01ba                               v5 = iconst.i32 0
;; @01c0                               v6 = bitcast.f64x2 little v2
;; @01c0                               v7 = bitcast.f64x2 little v2
;; @01c0                               v8 = fcmp ne v6, v7
;; @01c2                               v9 = uextend.i64 v5  ; v5 = 0
;; @01c2                               v10 = global_value.i64 gv4
;; @01c2                               v11 = iadd v10, v9
;; @01c2                               store little heap v8, v11
;; @01c6                               jump block1
;;
;;                                 block1:
;; @01c6                               return
;; }
;;
;; function u0:26(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @01c8                               v3 = global_value.i64 gv3
;; @01c8                               v4 = load.i64 notrap aligned v3+8
;; @01c9                               v5 = iconst.i32 0
;; @01cf                               v6 = bitcast.f32x4 little v2
;; @01cf                               v7 = bitcast.f32x4 little v2
;; @01cf                               v8 = fcmp lt v6, v7
;; @01d1                               v9 = uextend.i64 v5  ; v5 = 0
;; @01d1                               v10 = global_value.i64 gv4
;; @01d1                               v11 = iadd v10, v9
;; @01d1                               store little heap v8, v11
;; @01d5                               jump block1
;;
;;                                 block1:
;; @01d5                               return
;; }
;;
;; function u0:27(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @01d7                               v3 = global_value.i64 gv3
;; @01d7                               v4 = load.i64 notrap aligned v3+8
;; @01d8                               v5 = iconst.i32 0
;; @01de                               v6 = bitcast.f64x2 little v2
;; @01de                               v7 = bitcast.f64x2 little v2
;; @01de                               v8 = fcmp lt v6, v7
;; @01e0                               v9 = uextend.i64 v5  ; v5 = 0
;; @01e0                               v10 = global_value.i64 gv4
;; @01e0                               v11 = iadd v10, v9
;; @01e0                               store little heap v8, v11
;; @01e4                               jump block1
;;
;;                                 block1:
;; @01e4                               return
;; }
;;
;; function u0:28(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @01e6                               v3 = global_value.i64 gv3
;; @01e6                               v4 = load.i64 notrap aligned v3+8
;; @01e7                               v5 = iconst.i32 0
;; @01ed                               v6 = bitcast.f32x4 little v2
;; @01ed                               v7 = bitcast.f32x4 little v2
;; @01ed                               v8 = fcmp le v6, v7
;; @01ef                               v9 = uextend.i64 v5  ; v5 = 0
;; @01ef                               v10 = global_value.i64 gv4
;; @01ef                               v11 = iadd v10, v9
;; @01ef                               store little heap v8, v11
;; @01f3                               jump block1
;;
;;                                 block1:
;; @01f3                               return
;; }
;;
;; function u0:29(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @01f5                               v3 = global_value.i64 gv3
;; @01f5                               v4 = load.i64 notrap aligned v3+8
;; @01f6                               v5 = iconst.i32 0
;; @01fc                               v6 = bitcast.f64x2 little v2
;; @01fc                               v7 = bitcast.f64x2 little v2
;; @01fc                               v8 = fcmp le v6, v7
;; @01fe                               v9 = uextend.i64 v5  ; v5 = 0
;; @01fe                               v10 = global_value.i64 gv4
;; @01fe                               v11 = iadd v10, v9
;; @01fe                               store little heap v8, v11
;; @0202                               jump block1
;;
;;                                 block1:
;; @0202                               return
;; }
;;
;; function u0:30(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @0204                               v3 = global_value.i64 gv3
;; @0204                               v4 = load.i64 notrap aligned v3+8
;; @0205                               v5 = iconst.i32 0
;; @020b                               v6 = bitcast.f32x4 little v2
;; @020b                               v7 = bitcast.f32x4 little v2
;; @020b                               v8 = fcmp gt v6, v7
;; @020d                               v9 = uextend.i64 v5  ; v5 = 0
;; @020d                               v10 = global_value.i64 gv4
;; @020d                               v11 = iadd v10, v9
;; @020d                               store little heap v8, v11
;; @0211                               jump block1
;;
;;                                 block1:
;; @0211                               return
;; }
;;
;; function u0:31(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @0213                               v3 = global_value.i64 gv3
;; @0213                               v4 = load.i64 notrap aligned v3+8
;; @0214                               v5 = iconst.i32 0
;; @021a                               v6 = bitcast.f64x2 little v2
;; @021a                               v7 = bitcast.f64x2 little v2
;; @021a                               v8 = fcmp gt v6, v7
;; @021c                               v9 = uextend.i64 v5  ; v5 = 0
;; @021c                               v10 = global_value.i64 gv4
;; @021c                               v11 = iadd v10, v9
;; @021c                               store little heap v8, v11
;; @0220                               jump block1
;;
;;                                 block1:
;; @0220                               return
;; }
;;
;; function u0:32(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @0222                               v3 = global_value.i64 gv3
;; @0222                               v4 = load.i64 notrap aligned v3+8
;; @0223                               v5 = iconst.i32 0
;; @0229                               v6 = bitcast.f32x4 little v2
;; @0229                               v7 = bitcast.f32x4 little v2
;; @0229                               v8 = fcmp ge v6, v7
;; @022b                               v9 = uextend.i64 v5  ; v5 = 0
;; @022b                               v10 = global_value.i64 gv4
;; @022b                               v11 = iadd v10, v9
;; @022b                               store little heap v8, v11
;; @022f                               jump block1
;;
;;                                 block1:
;; @022f                               return
;; }
;;
;; function u0:33(i64 vmctx, i64, i8x16) fast {
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
;;                                 block0(v0: i64, v1: i64, v2: i8x16):
;; @0231                               v3 = global_value.i64 gv3
;; @0231                               v4 = load.i64 notrap aligned v3+8
;; @0232                               v5 = iconst.i32 0
;; @0238                               v6 = bitcast.f64x2 little v2
;; @0238                               v7 = bitcast.f64x2 little v2
;; @0238                               v8 = fcmp ge v6, v7
;; @023a                               v9 = uextend.i64 v5  ; v5 = 0
;; @023a                               v10 = global_value.i64 gv4
;; @023a                               v11 = iadd v10, v9
;; @023a                               store little heap v8, v11
;; @023e                               jump block1
;;
;;                                 block1:
;; @023e                               return
;; }
