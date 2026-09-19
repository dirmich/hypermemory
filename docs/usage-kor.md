# Hyper Memory — 사용 설명서 (Usage Guide)

본 문서는 **Hyper Memory**의 CLI 명령어 사용법, REST API 연동 규격, AI 에이전트를 위한 MCP(Model Context Protocol) 설정법, 그리고 옵시디언 볼트(Obsidian Vault) 양방향 동기화 방법을 안내하는 상세 매뉴얼입니다.

---

## 1. 빠른 설치 및 실행

### 필수 요구사항
- **Rust 툴체인** (1.80 이상): [rustup.rs](https://rustup.rs/)를 통해 설치 가능
- 지원 운영체제: **Windows**, **macOS**, **Linux** (완전한 크로스 플랫폼 지원)

### 옵션 A: 사전 빌드 바이너리 다운로드 (설치 불필요)
[GitHub Releases](https://github.com/dirmich/hypermemory/releases)에서 OS별 독립 실행파일을 다운로드하여 즉시 실행할 수 있습니다:
- **macOS (Apple Silicon)**: `hyper-memory-v0.1.7-macos-aarch64.tar.gz`
- **Linux / Windows**: Releases 페이지의 아카이브 다운로드

```bash
# macOS / Linux
tar -xvf hyper-memory-*.tar.gz
chmod +x hyper-memory
./hyper-memory doctor

# Windows (PowerShell)
Expand-Archive -Path hyper-memory-*.zip -DestinationPath .
.\hyper-memory.exe doctor
```

### 옵션 B: 소스코드에서 빌드
```bash
git clone https://github.com/dirmich/hypermemory.git
cd hypermemory
cargo build --release
```

빌드 완료 후 생성 위치:
- **macOS / Linux**: `target/release/hyper-memory`
- **Windows**: `target/release/hyper-memory.exe`

시스템 PATH에 등록하려면:
```bash
cargo install --path .
```

---

## 2. CLI 명령어 레퍼런스

Hyper Memory는 단일 통합 바이너리 `hyper-memory`로 모든 기능을 제어합니다.

### 2.1 공통 글로벌 옵션
- `-d, --data-dir <경로>`: SQLite 데이터베이스 및 인덱스 저장 경로 (기본값: `./.hyper-memory`)
- `-v, --vault-path <경로>`: 옵시디언 볼트 디렉토리 경로 (기본값: `./HyperMemoryVault`)

---

### 2.2 정보 및 결정사항 수집 (`add`)
대화 내용, 사실, 중요한 결정사항을 컨테이너(워크스페이스/프로젝트 네임스페이스)에 수집·추출합니다:

```bash
# 기본(default) 컨테이너에 수집
hyper-memory add "안정성과 메모리 효율을 위해 Hyper Memory 코어 엔진을 Rust로 개발하기로 결정함."

# 특정 프로젝트 네임스페이스에 수집
hyper-memory add --container "my-project" "분석용 데이터베이스를 PostgreSQL에서 ClickHouse로 이전하기로 결정함."
```

출력 예시:
```text
✅ Ingested document: doc_8a8d091474f6aaf7
   Extracted 1 memories into container 'my-project'.
   - [Decision] 분석용 데이터베이스를 PostgreSQL에서 ClickHouse로 이전하기로 결정함
```

---

### 2.3 하이브리드 검색 (`search`)
BM25 어휘 검색, INT8 양자화 벡터 검색, 최신성(Recency), 중요도, 시간 관계를 결합한 통합 하이브리드 검색을 수행합니다:

```bash
hyper-memory search --container "my-project" "데이터베이스"
```

출력 예시:
```text
🔍 Search results for "데이터베이스" (Container: my-project):
   1. [0.512] [Decision] 분석용 데이터베이스를 PostgreSQL에서 ClickHouse로 이전하기로 결정함
```

옵션:
- `-c, --container <이름>`: 컨테이너 범위 (기본값: `default`)
- `-l, --limit <개수>`: 최대 반환 결과 수 (기본값: `5`)

---

### 2.4 에이전트용 압축 컨텍스트 생성 (`context`)
AI 에이전트(LLM) 시스템 프롬프트에 바로 주입할 수 있도록, 설정된 토큰 예산(기본 2,000토큰) 내에서 현재 유효한 핵심 사실만 깔끔한 마크다운으로 컴파일합니다:

```bash
hyper-memory context --container "my-project" "현재 데이터베이스는 무엇인가요?" --max-tokens 1500
```

출력 예시:
```markdown
# Hyper Memory Context (Budget: 1500 tokens, Used: ~25 tokens)

## Relevant Current Memories
- [Decision] 분석용 데이터베이스를 PostgreSQL에서 ClickHouse로 이전하기로 결정함 (conf: 0.95, imp: 0.90)

## Sources & Provenance
- Memory `mem_1603d84ad1dc3a07` -> Source `doc_8a8d091474f6aaf7`
```

---

### 2.5 옵시디언 볼트 양방향 동기화 (`wiki-sync`)
저장소의 기억 그래프를 사람이 읽을 수 있는 옵시디언 마크다운으로 내보내고, 사람이 옵시디언에서 수기 작성한 메모나 태그를 기억으로 역방향 가져옵니다:

```bash
hyper-memory wiki-sync --container "my-project" --vault-path ~/Documents/ObsidianVault
```

출력 예시:
```text
🔄 Obsidian Vault Synchronization Complete:
   - Vault path: "/Users/.../Documents/ObsidianVault"
   - Exported pages: 4
   - Imported memories from vault notes: 2
   - Control tags processed: 1
```

---

### 2.6 시스템 진단 (`doctor`)
저장소 파일 무결성, SQLite 연결 상태, 문서 및 메모리 적재 수량을 확인합니다:

```bash
hyper-memory doctor
```

---

### 2.7 벤치마크 실행 (`benchmark`)
내장 벤치마크를 구동하여 수집 속도, 중복 배제율, 하이브리드 검색 지연시간, 사실 갱신 정합성을 검증합니다:

```bash
hyper-memory benchmark
```

---

### 2.8 REST API 서버 실행 (`start`)
Axum 기반 REST API 웹 서버를 백그라운드 또는 포그라운드로 실행합니다:

```bash
hyper-memory start --port 6767
```

---

### 2.9 MCP 서버 실행 (`mcp`)
Claude Desktop, Claude Code, Cursor, OpenCode 등과 연동하기 위한 표준 MCP stdio 서버를 실행합니다:

```bash
hyper-memory mcp
```

---

## 3. REST API 연동 가이드

기본 주소: `http://localhost:6767`

### 3.1 문서 및 대화 수집 (Ingest Document)
- **메서드/경로**: `POST /v1/documents`
- **요청 Body**:
```json
{
  "container": "project_alpha",
  "content": "대규모 분석 트래픽 처리를 위해 ClickHouse를 도입하기로 결정함.",
  "title": "DB 아키텍처 결정",
  "source_type": "conversation"
}
```
- **응답 Body**:
```json
{
  "document_id": "doc_2f8a1c90",
  "is_duplicate": false,
  "chunk_count": 1,
  "extracted_memories": 1
}
```

---

### 3.2 하이브리드 검색 (Hybrid Search)
- **메서드/경로**: `POST /v1/search`
- **요청 Body**:
```json
{
  "container": "project_alpha",
  "query": "분석용 DB는 무엇을 쓰는가?",
  "limit": 5
}
```
- **응답 Body**:
```json
{
  "query": "분석용 DB는 무엇을 쓰는가?",
  "hits": [
    {
      "id": "mem_4a8b7c",
      "text": "대규모 분석 트래픽 처리를 위해 ClickHouse를 도입하기로 결정함",
      "memory_type": "decision",
      "score": 0.584,
      "match_sources": ["lexical", "semantic"]
    }
  ]
}
```

---

### 3.3 에이전트 프롬프트 컨텍스트 생성 (Compile Agent Context)
- **메서드/경로**: `POST /v1/context`
- **요청 Body**:
```json
{
  "container": "project_alpha",
  "query": "현재 데이터베이스 아키텍처",
  "max_tokens": 1500
}
```
- **응답 Body**:
```json
{
  "context": "# Hyper Memory Context ...",
  "query": "현재 데이터베이스 아키텍처"
}
```

---

### 3.4 기억 목록 조회 (List Memories)
- **메서드/경로**: `GET /v1/memories?container=project_alpha`

---

### 3.5 프로필 조회 (Get Profile)
- **메서드/경로**: `GET /v1/profile/project_alpha`

---

### 3.6 옵시디언 볼트 동기화 (Wiki Sync)
- **메서드/경로**: `POST /v1/wiki/sync`
- **요청 Body**:
```json
{
  "container": "project_alpha"
}
```

---

### 3.7 헬스체크 (Health Check)
- **메서드/경로**: `GET /health`
- **응답**: `200 OK`

---

## 4. MCP (Model Context Protocol) AI 에이전트 연동

Hyper Memory는 Anthropic의 표준 MCP 프로토콜을 구현하여 **Claude Code**, **Claude Desktop**, **Cursor**, **OpenCode** 등과 즉시 연동됩니다.

### 4.1 Claude Desktop 설정
`claude_desktop_config.json`에 다음 설정을 추가합니다:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "hypermemory": {
      "command": "/절대경로/hyper-memory",
      "args": [
        "mcp",
        "--data-dir", "/절대경로/.hyper-memory",
        "--vault-path", "/절대경로/HyperMemoryVault"
      ]
    }
  }
}
```

### 4.2 Claude Code CLI 연동
터미널에서 다음 명령을 실행하여 도구를 자동 등록합니다:
```bash
claude mcp add hypermemory -- /절대경로/hyper-memory mcp
```

### 4.3 제공되는 11개 MCP 도구 목록
| 도구 이름 | 파라미터 | 설명 |
|---|---|---|
| `memory_add` | `container`, `text` | 대화나 사실을 수집하고 기억으로 추출 |
| `memory_search` | `container`, `query`, `limit` | 시간과 의미를 고려한 하이브리드 검색 |
| `memory_get` | `id` | 특정 기억 객체 상세 정보 조회 |
| `memory_update` | `id`, `canonical_text`, `is_pinned` | 기억 텍스트 갱신 또는 고정(Pin) 설정 |
| `memory_forget` | `id` | 기억을 아카이브로 이동하여 잊기 처리 |
| `profile_get` | `container` | 사용자/프로젝트 프로필 및 선호도 KV 조회 |
| `wiki_search` | `container`, `query` | 옵시디언 볼트 마크다운 문서 검색 |
| `wiki_get_page` | `folder`, `title` | 옵시디언 위키 페이지 원문 마크다운 조회 |
| `wiki_update_page`| `container` | 옵시디언 볼트 양방향 동기화 트리거 |
| `project_context`| `container`, `query`, `max_tokens` | 토큰 예산 내에서 에이전트용 압축 프롬프트 생성 |
| `source_get` | `document_id` | 기억의 근거가 된 원본 문서 및 청크 조회 |

---

## 5. 옵시디언 볼트(Obsidian Vault) 구조 및 양방향 동기화

Hyper Memory는 기계의 기억 그래프를 사람이 읽기 좋은 마크다운 문서로 변환합니다.

### 5.1 볼트 폴더 구조
```
HyperMemoryVault/
├── Home.md                     # 대시보드 인덱스
├── People/                     # 인물 정보
│   └── Alice.md
├── Projects/                   # 프로젝트 정보
│   └── Hyper Memory.md
├── Technologies/               # 기술 스택
│   ├── Rust.md
│   └── ClickHouse.md
├── Decisions/                  # 아키텍처 및 비즈니스 결정사항
│   └── Database Migration.md
├── Topics/                     # 일반 주제
├── Daily/                      # 자동 일간 노트
│   └── 2026-09-19.md
└── Sources/                    # 원본 출처 인덱스
```

### 5.2 영역 분리 및 사용자 메모 100% 보존
생성되는 모든 마크다운 파일은 AI 관리 영역과 사용자 메모 영역이 철저히 구분됩니다:

```markdown
---
hyper_memory_id: ent_931e87a4a1e4c737
type: Technology
updated: 2026-09-19
managed: true
---

# Rust

<!-- HYPER_MEMORY:AUTO:BEGIN -->
## Summary
Rust에 대한 요약 및 현황입니다.

## Decisions
- Hyper Memory 코어 엔진을 Rust로 개발하기로 결정함

## Sources
- [[Sources/doc_8a8d091474f6aaf7]]
<!-- HYPER_MEMORY:AUTO:END -->

## My Notes
(이 영역에 개인 메모, 생각, 의견을 자유롭게 작성하세요!
AI가 동기화할 때 AUTO 블록만 갱신하며 이 My Notes 내용은 절대 유실되지 않고 100% 보존됩니다.)
```

### 5.3 옵시디언 제어 태그
옵시디언 노트 안 어디서든 다음 태그를 작성하면 AI 기억 엔진이 이를 감지하여 처리합니다:
- `#hypermemory/pin`: 기억을 **Hot** 계층에 영구 고정하여 시간 감쇠나 아카이빙을 방지합니다.
- `#hypermemory/archive`: 해당 기억을 Cold/Archive 보관소로 즉시 이동합니다.
- `#hypermemory/no-ai`: 해당 노트나 문장을 AI 수집 및 인덱싱 대상에서 영구 제외합니다.

---

## 6. 서비스 배포 모범 사례

프로덕션이나 장기 실행 환경에서는 아래와 같이 디렉토리를 배치하는 것을 권장합니다:

```bash
~/.hyper-memory/
├── hypermemory.db          # 메인 SQLite 데이터베이스
├── hypermemory.db-wal      # WAL 트랜잭션 로그
└── HyperMemoryVault/       # 동기화할 옵시디언 볼트 폴더
```

서버 백그라운드 구동 (예: systemd 서비스 또는 터미널 백그라운드):
```bash
hyper-memory start --port 6767 --data-dir ~/.hyper-memory --vault-path ~/.hyper-memory/HyperMemoryVault
```
