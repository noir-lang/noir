#include <stdio.h>
#include <string>
#include <sys/resource.h>
#include <sys/time.h>
#include <time.h>

class Timer {
  private:
    struct timespec _startTime;
    struct timespec _endTime;

  public:
    Timer()
        : _endTime({})
    {
        start();
    }

    void start() { clock_gettime(CLOCK_REALTIME, &_startTime); }

    void end() { clock_gettime(CLOCK_REALTIME, &_endTime); }

    std::string toString() const
    {
        struct timespec endTime;
        if (_endTime.tv_nsec == 0 && _endTime.tv_sec == 0) {
            clock_gettime(CLOCK_REALTIME, &endTime);
        } else {
            endTime = _endTime;
        }

        long seconds = endTime.tv_sec - _startTime.tv_sec;
        long ns = endTime.tv_nsec - _startTime.tv_nsec;

        if (_startTime.tv_nsec > endTime.tv_nsec) { // clock underflow
            --seconds;
            ns += 1000000000;
        }

        return std::to_string((double)seconds + (double)ns / (double)1000000000);
    }
};