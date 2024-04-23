;;! target = "s390x"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -O static-memory-forced -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i32 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store offset=0)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load offset=0))

;; wasm[0]::function[0]:
;;       lg      %r1, 8(%r2)
;;       lg      %r1, 0(%r1)
;;       la      %r1, 0xa0(%r1)
;;       clgrtle %r15, %r1
;;       stmg    %r7, %r15, 0x38(%r15)
;;       lgr     %r1, %r15
;;       aghi    %r15, -0xa0
;;       stg     %r1, 0(%r15)
;;       lgr     %r14, %r2
;;       llgfr   %r3, %r4
;;       lghi    %r2, 0
;;       lgr     %r7, %r14
;;       lgr     %r4, %r3
;;       ag      %r4, 0x60(%r7)
;;       clgfi   %r3, 0xfffffffc
;;       locgrh  %r4, %r2
;;       strv    %r5, 0(%r4)
;;       lmg     %r7, %r15, 0xd8(%r15)
;;       br      %r14
;;
;; wasm[0]::function[1]:
;;       lg      %r1, 8(%r2)
;;       lg      %r1, 0(%r1)
;;       la      %r1, 0xa0(%r1)
;;       clgrtle %r15, %r1
;;       stmg    %r14, %r15, 0x70(%r15)
;;       lgr     %r1, %r15
;;       aghi    %r15, -0xa0
;;       stg     %r1, 0(%r15)
;;       llgfr   %r3, %r4
;;       lghi    %r5, 0
;;       lgr     %r4, %r3
;;       ag      %r4, 0x60(%r2)
;;       clgfi   %r3, 0xfffffffc
;;       locgrh  %r4, %r5
;;       lrv     %r2, 0(%r4)
;;       lmg     %r14, %r15, 0x110(%r15)
;;       br      %r14
