#!/bin/sh

rm output/target/etc/init.d/S01syslogd || true >/dev/null
rm output/target/etc/init.d/S02klogd || true >/dev/null
rm output/target/etc/init.d/S02sysctl || true >/dev/null
rm output/target/etc/init.d/S20seedrng || true >/dev/null
rm output/target/etc/init.d/S40network || true >/dev/null
cp ../config/inittab output/target/etc
cp ../config/busybox output/target/bin
cp ../config/init output/target/

