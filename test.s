.data
hello: .string "hello world\n"

.text
.global _start
_start:
    li a0, 0x10000000
    la a1, hello

    # h
    lbu a2, 0(a1)
    sb a2, 0(a0)

    # e
    lbu a2, 1(a1)
    sb a2, 0(a0)

    # l
    lbu a2, 2(a1)
    sb a2, 0(a0)

    # l
    lbu a2, 3(a1)
    sb a2, 0(a0)

    # o
    lbu a2, 4(a1)
    sb a2, 0(a0)

    # space
    lbu a2, 5(a1)
    sb a2, 0(a0)

    # w
    lbu a2, 6(a1)
    sb a2, 0(a0)

    # o
    lbu a2, 7(a1)
    sb a2, 0(a0)

    # r
    lbu a2, 8(a1)
    sb a2, 0(a0)

    # l
    lbu a2, 9(a1)
    sb a2, 0(a0)

    # d
    lbu a2, 10(a1)
    sb a2, 0(a0)

    # \n
    lbu a2, 11(a1)
    sb a2, 0(a0)
