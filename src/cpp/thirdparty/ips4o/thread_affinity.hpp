#pragma once

#include <numa.h>

#include <cassert>

namespace pinning {

class Uniform {
 public:
    static int getNodeId(int tid, int num_threads) {
        // Assume each numa node has the same thread capacity
        int numaNodes = std::max(numa_num_configured_nodes(), 1);
        int threadsPerNode = (num_threads + numaNodes - 1) / (numaNodes);
        int left = num_threads % numaNodes;

        int nodeId = tid / threadsPerNode;

        if (nodeId >= left && left) {
            assert(threadsPerNode > 1);
            nodeId = left + (tid - threadsPerNode * left) / (threadsPerNode - 1);
        }

        return nodeId;
    }

    static bool isLeadingThread(int tid, int num_threads) {
        return !tid || getNodeId(tid, num_threads) != getNodeId(tid - 1, num_threads);
    }

    static void setThreadAffinity(int tid, int num_threads) {
        const int nodeId = getNodeId(tid, num_threads);
        numa_run_on_node(nodeId);
        numa_set_preferred(nodeId);
    }

    static void unsetThreadAffinity() {
        numa_run_on_node(-1);
        numa_set_preferred(-1);
    }

    static int getThreadCountOfNode(int nodeId, int num_threads) {
        int numaNodes = std::max(numa_num_configured_nodes(), 1);
        int perNode = num_threads / numaNodes;
        int left = num_threads % numaNodes;
        return perNode + (nodeId < left);
    }

    static std::string name() { return "Uniform"; }
};

class Greedy {
 public:
    static int getNodeId(int tid, int num_threads) {
        // Assume each numa node has the same thread capacity
        int totalCores = numa_num_configured_cpus();
        int numaNodes = std::max(numa_num_configured_nodes(), 1);
        int maxThreadsPerNode = totalCores / numaNodes;

        return tid / maxThreadsPerNode;
    }

    static bool isLeadingThread(int tid, int num_threads) {
        return !tid || getNodeId(tid, num_threads) != getNodeId(tid - 1, num_threads);
    }

    static void setThreadAffinity(int tid, int num_threads) {
        const int nodeId = getNodeId(tid, num_threads);
        numa_run_on_node(nodeId);
        numa_set_preferred(nodeId);
    }

    static void unsetThreadAffinity() {
        numa_run_on_node(-1);
        numa_set_preferred(-1);
    }

    static int getThreadCountOfNode(int nodeId, int num_threads) {
        int numaNodes = std::max(numa_num_configured_nodes(), 1);

        int totalCores = numa_num_configured_cpus();
        int maxThreadsPerNode = totalCores / numaNodes;
        int fullNodes = num_threads / maxThreadsPerNode;

        if (nodeId < fullNodes)
            return maxThreadsPerNode;
        else if (nodeId == fullNodes)
            return num_threads % maxThreadsPerNode;
        else
            return 0;
    }
    static std::string name() { return "Greedy"; }
};

class Default {
 public:
    static int getNodeId(int, int) { return 0; }

    static bool isLeadingThread(int tid, int) { return tid == 0; }

    static void setThreadAffinity(int, int) {}

    static void unsetThreadAffinity() {}

    static int getThreadCountOfNode(int, int num_threads) { return num_threads; }
    static std::string name() { return "Default"; }
};
}  // namespace pinning

void pin() {
#pragma omp parallel
    {
        int my_id = omp_get_thread_num();
        int threads = omp_get_num_threads();
        pinning::Uniform::setThreadAffinity(my_id, threads);
    }
}

void unpin() {
#pragma omp parallel
    {
        int my_id = omp_get_thread_num();
        int threads = omp_get_num_threads();
        pinning::Uniform::setThreadAffinity(my_id, threads);
        pinning::Uniform::unsetThreadAffinity();
    }
}

template <typename Strategy>
void pinByStrategy() {
#pragma omp parallel
    {
        int my_id = omp_get_thread_num();
        int threads = omp_get_num_threads();
        Strategy::setThreadAffinity(my_id, threads);
    }
}

template <typename Strategy>
void unpinByStrategy() {
#pragma omp parallel
    {
        int my_id = omp_get_thread_num();
        int threads = omp_get_num_threads();
        Strategy::setThreadAffinity(my_id, threads);
        Strategy::unsetThreadAffinity();
    }
}
