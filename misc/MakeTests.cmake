set(BITS 32)
set(ARCH rv32ima_zicsr_zifencei)
set(ABI ilp32)
set(RVTEST_FOLDER riscv-tests)
set(TESTBINS_FOLDER testbins)
set(MISC_FOLDER misc)
set(CURRENT_EXECUTABLE "${CMAKE_CURRENT_LIST_DIR}/MakeTests.cmake")
set(DIST "unknown-elf")

function (build_asm asm_path out_path)
    file(MAKE_DIRECTORY "${out_path}/bin")
    file(MAKE_DIRECTORY "${out_path}/dumped")

    file(GLOB files ${asm_path})
    foreach(file ${files})
        get_filename_component(filename ${file} NAME_WE)
        set(filename_bin "${filename}.bin")
        set(filename_dump "${filename}.dump")

        exec_program("/home/bane9/Desktop/riscv/bin/riscv${BITS}-${DIST}-gcc -T${MISC_FOLDER}/link.ld -I${RVTEST_FOLDER}/env/p -I${RVTEST_FOLDER}/isa/macros/scalar -nostdlib -ffreestanding -march=${ARCH} -mabi=${ABI} -nostartfiles -O0 -o temp ${file}")
        exec_program("/home/bane9/Desktop/riscv/bin/riscv${BITS}-${DIST}-objcopy -O binary temp ${out_path}/bin/${filename_bin}")
        exec_program("/home/bane9/Desktop/riscv/bin/riscv${BITS}-${DIST}-objdump --disassemble-all -Mno-aliases temp > ${out_path}/dumped/${filename_dump}")
    endforeach()
endfunction()

get_filename_component(CURRENT_DIR ${CMAKE_CURRENT_SOURCE_DIR} ABSOLUTE)

string(REGEX MATCH ".*${MISC_FOLDER}$" IS_MISC_DIR ${CURRENT_DIR})

if(IS_MISC_DIR)
    message("Current directory is 'misc'. Changing to parent directory.")
    execute_process(
        COMMAND ${CMAKE_COMMAND} -P ${CURRENT_EXECUTABLE}
        WORKING_DIRECTORY ${CURRENT_DIR}/..
    )
else()
    if(WIN32)
        exec_program("rmdir /s /q ${TESTBINS_FOLDER}")
    else()
        exec_program("rm -rf ${TESTBINS_FOLDER}")
    endif()

    file(MAKE_DIRECTORY ${TESTBINS_FOLDER})

    build_asm("${RVTEST_FOLDER}/isa/rv${BITS}ui/*.S" "${TESTBINS_FOLDER}/rv${BITS}ui")
    build_asm("${RVTEST_FOLDER}/isa/rv${BITS}um/*.S" "${TESTBINS_FOLDER}/rv${BITS}um")
    build_asm("${RVTEST_FOLDER}/isa/rv${BITS}ua/*.S" "${TESTBINS_FOLDER}/rv${BITS}ua")

    build_asm("${RVTEST_FOLDER}/isa/rv${BITS}mi/*.S" "${TESTBINS_FOLDER}/rv${BITS}mi")
    build_asm("${RVTEST_FOLDER}/isa/rv${BITS}si/*.S" "${TESTBINS_FOLDER}/rv${BITS}si")

    file(REMOVE temp)
endif()
