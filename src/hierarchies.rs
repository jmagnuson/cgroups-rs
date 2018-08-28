use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;
use std::path::{Path, PathBuf};

use {Controllers, Hierarchy, Subsystem};
use ::pid::PidController;
use ::memory::MemController;
use ::cpuset::CpuSetController;
use ::cpuacct::CpuAcctController;
use ::cpu::CpuController;
use ::freezer::FreezerController;
use ::devices::DevicesController;
use ::net_cls::NetClsController;
use ::blkio::BlkIoController;
use ::perf_event::PerfEventController;
use ::net_prio::NetPrioController;
use ::hugetlb::HugeTlbController;
use ::rdma::RdmaController;

pub struct V1 {
    mount_point: String,
}

impl Hierarchy for V1 {
    fn subsystems(self: &Self) -> Vec<Subsystem> {
        let mut subs = vec![];
        if self.check_support(Controllers::Pids) {
            subs.push(Subsystem::Pid(PidController::new(self.root())));
        }
        if self.check_support(Controllers::Mem) {
            subs.push(Subsystem::Mem(MemController::new(self.root())));
        }
        if self.check_support(Controllers::CpuSet) {
            subs.push(Subsystem::CpuSet(CpuSetController::new(self.root())));
        }
        if self.check_support(Controllers::CpuAcct) {
            subs.push(Subsystem::CpuAcct(CpuAcctController::new(self.root())));
        }
        if self.check_support(Controllers::Cpu) {
            subs.push(Subsystem::Cpu(CpuController::new(self.root())));
        }
        if self.check_support(Controllers::Devices) {
            subs.push(Subsystem::Devices(DevicesController::new(self.root())));
        }
        if self.check_support(Controllers::Freezer) {
            subs.push(Subsystem::Freezer(FreezerController::new(self.root())));
        }
        if self.check_support(Controllers::NetCls) {
            subs.push(Subsystem::NetCls(NetClsController::new(self.root())));
        }
        if self.check_support(Controllers::BlkIo) {
            subs.push(Subsystem::BlkIo(BlkIoController::new(self.root())));
        }
        if self.check_support(Controllers::PerfEvent) {
            subs.push(Subsystem::PerfEvent(PerfEventController::new(self.root())));
        }
        if self.check_support(Controllers::NetPrio) {
            subs.push(Subsystem::NetPrio(NetPrioController::new(self.root())));
        }
        if self.check_support(Controllers::HugeTlb) {
            subs.push(Subsystem::HugeTlb(HugeTlbController::new(self.root())));
        }
        if self.check_support(Controllers::Rdma) {
            subs.push(Subsystem::Rdma(RdmaController::new(self.root())));
        }

        subs
    }

    fn check_support(self: &Self, sub: Controllers) -> bool {
        let root = self.root().read_dir().unwrap();
        for entry in root {
            if let Ok(entry) = entry {
                if entry.file_name().into_string().unwrap() == sub.to_string() {
                    return true;
                }
            }
        }
        return false;
    }

    fn root(self: &Self) -> PathBuf {
        PathBuf::from(self.mount_point.clone())
    }

    fn can_create_cgroup(self: &Self) -> bool {
        /*
         * V1 hierarchies do not support creating cgroups,
         * they have to be created in a subsystem
         */
        false
    }
}

impl V1 {
    pub fn new() -> Self {
        let mount_point = find_v1_mount().unwrap();
        V1 {
            mount_point: mount_point,
        }
    }
}

fn find_v1_mount() -> Option<String> {
    /* Open mountinfo so we can get a parseable mount list */
    let mountinfo_path = Path::new("/proc/self/mountinfo");

    /* If /proc isn't mounted, or something else happens, then bail out */
    if mountinfo_path.exists() == false {
        return None;
    }

    let mountinfo_file = File::open(mountinfo_path).unwrap();
    let mountinfo_reader = BufReader::new(&mountinfo_file);
    for _line in mountinfo_reader.lines() {
        let line = _line.unwrap();
        let mut fields = line.split_whitespace();
        let index = line.find(" - ").unwrap();
        let mut more_fields = line[index + 3..].split_whitespace().collect::<Vec<_>>();
        let fstype = more_fields[0];
        if fstype == "tmpfs" && more_fields[2].contains("ro") {
            let cgroups_mount = fields.nth(4).unwrap();
            println!("found cgroups at {:?}", cgroups_mount);
            return Some(cgroups_mount.to_string());
        }
    }

    None
}
