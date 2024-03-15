;;! target = "x86_64"
;;! test = "optimize"
;;! flags = "-O static-memory-maximum-size=0 -O dynamic-memory-guard-size=0xffff"

(module
  (memory (export "memory") 0)

  (func (export "loads") (param i32) (result i32 i32 i32)
    ;; Within the guard region.
    local.get 0
    i32.load offset=0
    ;; Also within the guard region, bounds check should GVN with previous.
    local.get 0
    i32.load offset=4
    ;; Outside the guard region, needs additional bounds checks.
    local.get 0
    i32.load offset=0x000fffff
  )

  ;; Same as above, but for stores.
  (func (export "stores") (param i32 i32 i32 i32)
    local.get 0
    local.get 1
    i32.store offset=0
    local.get 0
    local.get 2
    i32.store offset=4
    local.get 0
    local.get 3
    i32.store offset=0x000fffff
  )
)

;; function u0:0(i64 vmctx, i64, i32) -> i32, i32, i32 fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+88
;;     gv5 = load.i64 notrap aligned checked gv3+80
;;     sig0 = (i64 vmctx, i32 uext, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32):
;;                                     v6 -> v0
;;                                     v38 -> v0
;;                                     v39 -> v0
;;                                     v40 -> v0
;;                                     v41 -> v0
;;                                     v42 -> v0
;;                                     v43 -> v0
;; @0047                               v9 = load.i64 notrap aligned v0+88
;; @0047                               v11 = load.i64 notrap aligned checked v0+80
;; @0047                               v8 = uextend.i64 v2
;; @0047                               v10 = icmp ugt v8, v9
;; @0047                               v13 = iconst.i64 0
;; @0047                               v12 = iadd v11, v8
;; @0047                               v14 = select_spectre_guard v10, v13, v12  ; v13 = 0
;; @0047                               v15 = load.i32 little heap v14
;;                                     v3 -> v15
;; @004c                               v21 = iconst.i64 4
;; @004c                               v22 = iadd v12, v21  ; v21 = 4
;; @004c                               v24 = select_spectre_guard v10, v13, v22  ; v13 = 0
;; @004c                               v25 = load.i32 little heap v24
;;                                     v4 -> v25
;; @0051                               v27 = iconst.i64 0x0010_0003
;; @0051                               v28 = uadd_overflow_trap v8, v27, heap_oob  ; v27 = 0x0010_0003
;; @0051                               v30 = icmp ugt v28, v9
;; @0051                               v33 = iconst.i64 0x000f_ffff
;; @0051                               v34 = iadd v12, v33  ; v33 = 0x000f_ffff
;; @0051                               v36 = select_spectre_guard v30, v13, v34  ; v13 = 0
;; @0051                               v37 = load.i32 little heap v36
;;                                     v5 -> v37
;; @0056                               jump block1
;;
;;                                 block1:
;; @0056                               return v15, v25, v37
;; }
;;
;; function u0:1(i64 vmctx, i64, i32, i32, i32, i32) fast {
;;     gv0 = vmctx
;;     gv1 = load.i64 notrap aligned readonly gv0+8
;;     gv2 = load.i64 notrap aligned gv1
;;     gv3 = vmctx
;;     gv4 = load.i64 notrap aligned gv3+88
;;     gv5 = load.i64 notrap aligned checked gv3+80
;;     sig0 = (i64 vmctx, i32 uext, i32 uext, i32 uext) -> i32 uext system_v
;;     sig1 = (i64 vmctx, i32 uext, i32 uext) -> i32 uext system_v
;;     sig2 = (i64 vmctx, i32 uext) -> i32 uext system_v
;;     stack_limit = gv2
;;
;;                                 block0(v0: i64, v1: i64, v2: i32, v3: i32, v4: i32, v5: i32):
;;                                     v6 -> v0
;;                                     v35 -> v0
;;                                     v36 -> v0
;;                                     v37 -> v0
;;                                     v38 -> v0
;;                                     v39 -> v0
;;                                     v40 -> v0
;; @005d                               v9 = load.i64 notrap aligned v0+88
;; @005d                               v11 = load.i64 notrap aligned checked v0+80
;; @005d                               v8 = uextend.i64 v2
;; @005d                               v10 = icmp ugt v8, v9
;; @005d                               v13 = iconst.i64 0
;; @005d                               v12 = iadd v11, v8
;; @005d                               v14 = select_spectre_guard v10, v13, v12  ; v13 = 0
;; @005d                               store little heap v3, v14
;; @0064                               v20 = iconst.i64 4
;; @0064                               v21 = iadd v12, v20  ; v20 = 4
;; @0064                               v23 = select_spectre_guard v10, v13, v21  ; v13 = 0
;; @0064                               store little heap v4, v23
;; @006b                               v25 = iconst.i64 0x0010_0003
;; @006b                               v26 = uadd_overflow_trap v8, v25, heap_oob  ; v25 = 0x0010_0003
;; @006b                               v28 = icmp ugt v26, v9
;; @006b                               v31 = iconst.i64 0x000f_ffff
;; @006b                               v32 = iadd v12, v31  ; v31 = 0x000f_ffff
;; @006b                               v34 = select_spectre_guard v28, v13, v32  ; v13 = 0
;; @006b                               store little heap v5, v34
;; @0070                               jump block1
;;
;;                                 block1:
;; @0070                               return
;; }
