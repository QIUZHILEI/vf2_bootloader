.global entry
entry:
    csrr t0, mhartid
    slli t1, t0 ,12
    la t2, _stack_start
    sub t2, t2, t1
    mv sp, t2
    la a0, _end
    j rust_entry