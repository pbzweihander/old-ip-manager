# IP-Manager

서울대학교 동아리 [Bacchus](bacchus.snucse.org) 에서 사용하는 IP 목록 관리 Slack App 입니다.

## Installation

```
git clone https://github.com/pbzweihander/ip_manager.git
cargo build
cargo install
```

rust nightly에서 작성된 코드입니다.

## Usage

### Server

```
ip_manager /path/to/settings.toml
```

### Slack Client

```
/ip-add				# 새 IP를 추가하는 Dialog를 띄웁니다.
/ip-edit <ip>		# IP를 수정하는 Dialog를 띄웁니다.
/ip-get <ip>		# IP의 정보를 가져옵니다.
/ip-list <query>	# query의 내용을 IP 목록에서 검색해, 결과를 출력합니다.
/ip-del <ip>		# IP를 삭제합니다.
```

## Settings

### `settings.toml` 파일 형식

```
verification_token = "SLACK_APP_VERIFICATION_TOKEN"
api_token = "SLACK_APP_API_TOKEN"
data_path = "path/to/data/folder"
```

## data folder

data 폴더에는 IP의 정보가 <ip>.toml 형식으로 담기게 됩니다.

### <ip>.toml 파일 형식

```
ip = "IP"
domain = "DOMAIN" # optional
using = true | false
open_ports = []
description = "DESCRIPTION" # optional
```
