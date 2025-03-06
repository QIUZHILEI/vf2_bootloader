.global entry
entry:
    csrr a0, mhartid
    slli t0, a0 ,12
    la t1, _stack_start
    sub t1, t1, t0
    mv sp, t1
    j rust_entry