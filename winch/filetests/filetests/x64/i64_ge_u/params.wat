;;! target = "x86_64"

(module
    (func (param i64) (param i64) (result i32)
        (local.get 0)
        (local.get 1)
        (i64.ge_u)
    )
)
;;      	 55                   	push	rbp
;;      	 4889e5               	mov	rbp, rsp
;;      	 4d8b5e08             	mov	r11, qword ptr [r14 + 8]
;;      	 4d8b1b               	mov	r11, qword ptr [r11]
;;      	 4981c318000000       	add	r11, 0x18
;;      	 4939e3               	cmp	r11, rsp
;;      	 0f8730000000         	ja	0x4b
;;   1b:	 4883ec18             	sub	rsp, 0x18
;;      	 48897c2410           	mov	qword ptr [rsp + 0x10], rdi
;;      	 4889742408           	mov	qword ptr [rsp + 8], rsi
;;      	 4c893424             	mov	qword ptr [rsp], r14
;;      	 488b442408           	mov	rax, qword ptr [rsp + 8]
;;      	 488b4c2410           	mov	rcx, qword ptr [rsp + 0x10]
;;      	 4839c1               	cmp	rcx, rax
;;      	 b900000000           	mov	ecx, 0
;;      	 400f93c1             	setae	cl
;;      	 89c8                 	mov	eax, ecx
;;      	 4883c418             	add	rsp, 0x18
;;      	 5d                   	pop	rbp
;;      	 c3                   	ret	
;;   4b:	 0f0b                 	ud2	
