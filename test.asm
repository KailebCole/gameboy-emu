; Game Boy CPU opcode test ROM
SECTION "Header", ROM0[$100]
    jp Start

SECTION "Code", ROM0[$150]
Start:
    ; 8-bit loads
    ld a, $12
    ld b, $34
    ld c, $56
    ld d, $78
    ld e, $9A
    ld h, $BC
    ld l, $DE

    ld [hl], a
    inc hl
    ld [hl], b
    inc hl
    ld [hl], c
    inc hl
    ld [hl], d
    inc hl
    ld [hl], e
    inc hl
    ld [hl], h
    inc hl
    ld [hl], l

    ld a, [hl]
    ld b, a
    ld c, b
    ld d, c
    ld e, d
    ld h, e
    ld l, h

    ; 16-bit loads
    ld bc, $1234
    ld de, $5678
    ld hl, $9ABC
    ld sp, $C000

    push bc
    push de
    push hl
    pop hl
    pop de
    pop bc

    ; Arithmetic
    ld a, $10
    add a, $10
    adc a, $10
    sub $05
    sbc a, $01
    inc a
    dec a
    inc b
    dec b
    inc c
    dec c
    inc d
    dec d
    inc e
    dec e
    inc h
    dec h
    inc l
    dec l

    ; Logic
    and $F0
    or $0F
    xor $FF

    jp JumpTarget
    call CallTarget
    ret

    jr Jump1
Jump1:
    jr nz, Jump2
Jump2:
    jr z, Jump3
Jump3:
    jr nc, Jump4
Jump4:
    jr c, Jump5
Jump5:


    sla a
    sra a
    srl a
    swap a

    ; Bit operations (CB prefix)
    ld b, $01
    bit 0, b
    set 0, b
    res 0, b

    ; Jumps/Calls/Returns
    jp JumpTarget
    call CallTarget
    ret


    jr Next1
Next1:
    jr nz, Next2
Next2:
    jr z, Next3
Next3:
    jr nc, Next4
Next4:
    jr c, Next5
Next5:



    ; Write success value to RAM
    ld a, $42
    ld [$C100], a

; Provide targets for jump/call opcode tests
JumpTarget:
    nop
CallTarget:
    ret

    ; Infinite loop
EndLoop:
    jp EndLoop