;;! target = "x86_64"
;;! flags = ["has_bmi1"]

(module
    (func (result i64)
        (local $foo i64)

        (i64.const 2)
        (local.set $foo)

        (local.get $foo)
        (i64.ctz)
    )
)
;;      	 55                   	push	rbp
;;      	 4889e5               	mov	rbp, rsp
;;      	 4d8b5e08             	mov	r11, qword ptr [r14 + 8]
;;      	 4d8b1b               	mov	r11, qword ptr [r11]
;;      	 4981c310000000       	add	r11, 0x10
;;      	 4939e3               	cmp	r11, rsp
;;      	 0f872d000000         	ja	0x48
;;   1b:	 4883ec10             	sub	rsp, 0x10
;;      	 48c744240800000000   	
;; 				mov	qword ptr [rsp + 8], 0
;;      	 4c893424             	mov	qword ptr [rsp], r14
;;      	 48c7c002000000       	mov	rax, 2
;;      	 4889442408           	mov	qword ptr [rsp + 8], rax
;;      	 488b442408           	mov	rax, qword ptr [rsp + 8]
;;      	 f3480fbcc0           	tzcnt	rax, rax
;;      	 4883c410             	add	rsp, 0x10
;;      	 5d                   	pop	rbp
;;      	 c3                   	ret	
;;   48:	 0f0b                 	ud2	
