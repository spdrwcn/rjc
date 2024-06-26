## rjc

 - 从`redis`获取所有键值对并多线程写入`json csv`文件内，自动判断文件后缀写入对应格式文件

 ```
 ./maccsv -h
Usage: maccsv.exe [OPTIONS]

Options:
  -i, --ip-address <IP_ADDRESS>  [default: redis://127.0.0.1:6379/0]
  -p, --path <PATH>              [default: mac.csv]
  -h, --help                     Print help
  -V, --version                  Print version
  ```

```
❯ ./maccsv
aadasdsadsa: 值类型不匹配，已忽略
获取数据: 1001001组
用时: 6.9401276s

❯ ./maccsv -p mac.json
aadasdsadsa: 值类型不匹配，已忽略
获取数据: 1001001组
用时: 5.8616294s
```


### value格式必须为`json`格式，如下：

```
{
    "bluetooth_mac": "60:A5:E2:43:BE:48",
    "wired_mac": "04:BF:1B:65:ED:9A",
    "wireless_mac": "60:A5:E2:43:BE:44"
}
```

### 例如`key`为`BPB4BX3`，`value`为以上，则写入`json`后的文件为以下：

 - Execl表格可直接导入

```
{
  { "SN": "BPB4BX3",
    "bluetooth_mac": "60:A5:E2:43:BE:48",
    "wired_mac": "04:BF:1B:65:ED:9A",
    "wireless_mac": "60:A5:E2:43:BE:44"
  }
}
```

 - 多个如下:

```
[
  { "SN": "BPB4BX3",
    "bluetooth_mac": "60:A5:E2:43:BE:48",
    "wired_mac": "04:BF:1B:65:ED:9A",
    "wireless_mac": "60:A5:E2:43:BE:44"
  },
  { "SN": "BPB4BX3",
    "bluetooth_mac": "60:A5:E2:43:BE:48",
    "wired_mac": "04:BF:1B:65:ED:9A",
    "wireless_mac": "60:A5:E2:43:BE:44"
  },
  { "SN": "BPB4BX3",
    "bluetooth_mac": "60:A5:E2:43:BE:48",
    "wired_mac": "04:BF:1B:65:ED:9A",
    "wireless_mac": "60:A5:E2:43:BE:44"
  }
]
```


### 写入`csv`后的文件为以下：


|  SN   | wired_mac  | wireless_mac  | bluetooth_mac  |
|  ----  | ----  | ----  | ----  |
| BPB4BX3  | 04:BF:1B:65:ED:9A | 60:A5:E2:43:BE:44 | 60:A5:E2:43:BE:48 |