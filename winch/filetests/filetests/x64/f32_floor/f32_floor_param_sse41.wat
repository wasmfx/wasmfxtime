;;! target = "x86_64"
;;! flags = ["has_sse41"]

(module
    (func (param f32) (result f32)
        (local.get 0)
        (f32.floor)
    )
)
;;      	 55                   	push	rbp
;;      	 4889e5               	mov	rbp, rsp
;;      	 4d8b5e08             	mov	r11, qword ptr [r14 + 8]
;;      	 4d8b1b               	mov	r11, qword ptr [r11]
;;      	 4981c310000000       	add	r11, 0x10
;;      	 4939e3               	cmp	r11, rsp
;;      	 0f8720000000         	ja	0x3b
;;   1b:	 4883ec10             	sub	rsp, 0x10
;;      	 f30f1144240c         	movss	dword ptr [rsp + 0xc], xmm0
;;      	 4c893424             	mov	qword ptr [rsp], r14
;;      	 f30f1044240c         	movss	xmm0, dword ptr [rsp + 0xc]
;;      	 660f3a0ac001         	roundss	xmm0, xmm0, 1
;;      	 4883c410             	add	rsp, 0x10
;;      	 5d                   	pop	rbp
;;      	 c3                   	ret	
;;   3b:	 0f0b                 	ud2	
