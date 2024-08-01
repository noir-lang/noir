
# NOTE: intended to be ran from one's external computer, connecting to Aztec mainframe
# the benchmark runs with headless capture and then we copy the trace file and run tracy profiler
# This is thus only really useful internally at Aztec, sorry external folks. It can be easily tweaked
# however for any SSH setup, especially an ubuntu one, and of course if you are just tracing on the
# same machine you can use the normal interactive tracy workflow.
set -eux
USER=$1
BOX=$USER-box
BENCHMARK=${2:-ultra_plonk_bench}
COMMAND=${3:-./bin/$BENCHMARK}

ssh $BOX "
	set -eux ;
	! [ -d ~/tracy ] && git clone https://github.com/wolfpld/tracy ~/tracy ;
	cd ~/tracy/capture ;
	sudo apt-get install libdbus-1-dev libdbus-glib-1-dev ;
	mkdir -p build && cd build && cmake .. && make -j ;
	./tracy-capture -a 127.0.0.1 -f -o trace-$BENCHMARK & ;
	sleep 0.1 ;
	cd ~/aztec-packages/barretenberg/cpp/ ;
	cmake --preset tracy && cmake --build --preset tracy --parallel $BENCHMARK ;
	cd build-tracy ;
	ninja $BENCHMARK ;
	$COMMAND ;
"
! [ -d ~/tracy ] && git clone https://github.com/wolfpld/tracy ~/tracy
cd ~/tracy
cmake -B profiler/build -S profiler -DCMAKE_BUILD_TYPE=Release
cmake --build profiler/build --parallel
scp $BOX:/mnt/user-data/$USER/tracy/capture/build/trace-$BENCHMARK .
~/tracy/profiler/build/tracy-profiler trace-$BENCHMARK
