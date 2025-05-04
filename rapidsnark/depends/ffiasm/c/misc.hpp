#ifndef MISC_H
#define MISC_H

#ifdef USE_OPENMP
#include <omp.h>
#endif
#include <cstdint>
#include <cassert>
#include <vector>
#include <thread>
#include <atomic>
#include <mutex>
#include <condition_variable>
#include <functional>

uint32_t log2 (uint32_t value);

#ifdef _OPENMP

/**
 * This object is used to temporarily change the max number of omp threads.
 * When the object is destructed, the max threads is set to it's original value.
 */
class ThreadLimit {
public:
    ThreadLimit(uint32_t maxThreads):
        prev_max_threads(omp_get_max_threads())
    {
        omp_set_num_threads(maxThreads);
    }

    ~ThreadLimit() noexcept
    {
        omp_set_num_threads(prev_max_threads);
    }

private:
    uint32_t prev_max_threads;
};

#endif // _OPENMP

class ThreadWorker
{
    std::atomic<bool> stop;
    std::function<void()> task;
    std::mutex mutex;
    std::condition_variable occupied;
    std::condition_variable finished;
    std::thread thread;

    void worker() {

        while (true) {
            {
                std::unique_lock<std::mutex> lock(mutex);

                occupied.wait(lock, [this]{return task || stop;});

                if (stop) {
                    return;
                }

                task();
                task = nullptr;
            }
            finished.notify_one();
        }
    }

public:
    ThreadWorker() :
        stop(false),
        thread(&ThreadWorker::worker, this)
    {}

    ~ThreadWorker() {
        stop = true;
        occupied.notify_one();

        thread.join();
    }

    template<typename Func>
    void submit(Func func) {
        {
            std::lock_guard<std::mutex> lock(mutex);
            task = func;
        }
        occupied.notify_one();
    }

    void wait() {
        std::unique_lock<std::mutex> lock(mutex);

        finished.wait(lock, [this]{return !task;});
    }
};

class ThreadPool {
    uint64_t nThreads;
    std::vector<ThreadWorker> workers;

public:
    ThreadPool(uint64_t _nThreads = 0) :
        nThreads(_nThreads==0 ? defaultThreadCount() : _nThreads),
        workers(nThreads-1)
    { }

    static uint64_t defaultThreadCount() {
        const uint64_t n = std::thread::hardware_concurrency();

        return n == 0 ? 1 : n;
    }

    static ThreadPool& defaultPool() {
        static ThreadPool pool;

        return pool;
    }

    uint64_t getThreadCount() const {
        return nThreads;
    }

    static std::vector<uint64_t> divideWork(uint64_t elementCount, uint64_t threadCount) {

        if (elementCount <= threadCount) {
            return std::vector<uint64_t>(elementCount, 1);
        }

        const uint64_t jobSize = elementCount / threadCount;
        const uint64_t elementRest = elementCount % threadCount;

        std::vector<uint64_t> jobs(threadCount, jobSize);

        for (int64_t i = 0; i < elementRest; i++) {
            jobs[i] += 1;
        }

        return jobs;
    }

    template<typename Func>
    void parallelFor(int64_t begin, int64_t end, Func&& func) {

        if (begin >= end) {
            return;
        }

        const uint64_t elementCount = end - begin;
        const auto     jobs = divideWork(elementCount, nThreads);
        const int64_t  jobCount = jobs.size();
        uint64_t       jobBegin = begin;
        int64_t        k = 0;

        for (; k < jobCount - 1; k++) {
            const uint64_t jobEnd = jobBegin + jobs[k];

            workers[k].submit([=]{func(jobBegin, jobEnd, k);});

            jobBegin = jobEnd;
        }

        func(jobBegin, jobBegin + jobs[k], k);

        for (k = 0; k < jobCount - 1; k++) {
            workers[k].wait();
        }
    }

    template<typename Func>
    void parallelBlock(Func&& func) {
        int64_t k = 0;

        for (; k < nThreads - 1; k++) {
            workers[k].submit([=]{func(k, nThreads);});
        }

        func(k, nThreads);

        for (k = 0; k < nThreads - 1; k++) {
            workers[k].wait();
        }
    }
};

#endif // MISC_H
