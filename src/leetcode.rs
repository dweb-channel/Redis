pub mod test {
    use std::collections::{HashSet, VecDeque};

    #[test]
    fn test_3310() {
        let result = remaining_methods(4, 1, vec![vec![1, 2], vec![0, 1], vec![3, 2]]);
        assert_eq!(vec![0, 1, 2, 3], result);
    }

    fn remaining_methods(n: i32, k: i32, invocations: Vec<Vec<i32>>) -> Vec<i32> {
        let n = n as usize;
        let k = k as usize;

        // 构建调用图（有向图）
        let mut graph = vec![vec![]; n]; // graph[i] 表示方法 i 调用的方法列表
        let mut indegree = vec![0; n]; // 入度数组，表示每个方法被多少个其他方法调用

        for invocation in invocations {
            let a = invocation[0] as usize;
            let b = invocation[1] as usize;
            graph[a].push(b);
            indegree[b] += 1;
        }

        // 使用 BFS 从方法 k 开始找到所有可疑方法
        let mut suspicious = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(k);
        suspicious.insert(k);

        while let Some(current) = queue.pop_front() {
            for &neighbor in &graph[current] {
                if !suspicious.contains(&neighbor) {
                    suspicious.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        // 检查是否有可疑方法被非可疑方法调用
        for i in 0..n {
            if !suspicious.contains(&i) {
                for &called in &graph[i] {
                    if suspicious.contains(&called) {
                        // 可疑方法被非可疑方法调用，不能移除任何方法
                        return (0..n as i32).collect();
                    }
                }
            }
        }

        // 返回剩下的方法，即非可疑的方法
        (0..n as i32)
            .filter(|&i| !suspicious.contains(&(i as usize)))
            .collect()
    }

    #[test]
    fn test_134() {
        let result = can_complete_circuit(vec![1, 2, 3, 4, 5], vec![3, 4, 5, 1, 2]);
        assert_eq!(result, 3);
    }
    // 只有当前的加油站到达历史最大值的时候才有可能是潜在的起始加油站
    fn can_complete_circuit(gas: Vec<i32>, cost: Vec<i32>) -> i32 {
        let mut sum = 0;
        let mut min = i32::MAX;
        let mut min_index = -1;

        for i in 0..gas.len() {
            sum += gas[i] - cost[i];
            if sum < min && sum < 0 {
                min = sum;
                min_index = i as i32;
            }
        }

        if sum < 0 {
            -1
        } else {
            (min_index + 1) % gas.len() as i32
        }
    }

    fn min_speed_on_time(dist: Vec<i32>, hour: f64) -> i32 {
        for velocity in 1..10_000_000 {
            let mut arrival_time = 0.0;
            let mut iter = dist.iter().peekable();
            while let Some(item) = iter.next() {
                let travel_time = (*item as f64) / (velocity as f64);
                if iter.peek().is_none() {
                    arrival_time = travel_time + arrival_time;
                } else {
                    println!(
                        "arrival_time:{},item:{},velocity:{}",
                        arrival_time, item, velocity
                    );
                    if (arrival_time + travel_time) > hour {
                        arrival_time = 0.0;
                        break;
                    }
                    arrival_time = travel_time.ceil() + arrival_time;
                }
            }
            print!("{} <= {}", arrival_time, hour);
            if arrival_time != 0.0 && arrival_time <= hour {
                return velocity;
            };
        }
        return -1;
    }
}
