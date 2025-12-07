# DataCode WebSocket Server

WebSocket server for remote execution of DataCode code.

## Starting the Server

```bash
# Start on default address (127.0.0.1:8080)
datacode --websocket

# Start with host and port specified via flags
datacode --websocket --host 0.0.0.0 --port 8899

# Start on custom address via environment variable
DATACODE_WS_ADDRESS=0.0.0.0:3000 datacode --websocket

# Combination: flags take priority over environment variable
DATACODE_WS_ADDRESS=127.0.0.1:8080 datacode --websocket --host 0.0.0.0 --port 8899
# Result: server will start on 0.0.0.0:8899
```

## Protocol

### Connection

Connect to WebSocket server at address `ws://127.0.0.1:8080` (or specified address).

### Request Format

WebSocket server supports several request types. All requests must contain `type` field to specify operation type.

#### Code Execution

Send JSON message with type `execute` and `code` field:

```json
{
  "type": "execute",
  "code": "print('Hello, World!')"
}
```

**Backward Compatibility:** Old format without `type` field is also supported:

```json
{
  "code": "print('Hello, World!')"
}
```

#### Connecting to SMB Share

To connect to SMB (Samba/CIFS) share, use type `smb_connect`:

```json
{
  "type": "smb_connect",
  "ip": "192.168.1.100",
  "login": "username",
  "password": "password",
  "domain": "WORKGROUP",
  "share_name": "share_name"
}
```

**Parameters:**
- `ip` - IP address or name of SMB server
- `login` - username
- `password` - user password
- `domain` - domain (usually `WORKGROUP` or domain name, can be empty string)
- `share_name` - name of SMB share

**Response:**
```json
{
  "success": true,
  "message": "Successfully connected to SMB share 'share_name'",
  "error": null
}
```

### Response Format

Server will return JSON with execution result:

**Successful Execution:**
```json
{
  "success": true,
  "output": "Hello, World!\n",
  "error": null
}
```

**Execution Error:**
```json
{
  "success": false,
  "output": "",
  "error": "Error: variable 'x' is not defined"
}
```

## Usage Examples

### JavaScript/Node.js

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://127.0.0.1:8080');

ws.on('open', function open() {
    const request = {
        type: "execute",
        code: "print('Hello from WebSocket!')"
    };
    ws.send(JSON.stringify(request));
});

ws.on('message', function message(data) {
    const response = JSON.parse(data);
    console.log('Output:', response.output);
    if (response.error) {
        console.error('Error:', response.error);
    }
});
```

### Python

```python
import asyncio
import websockets
import json

async def execute_code():
    uri = "ws://127.0.0.1:8080"
    async with websockets.connect(uri) as websocket:
        request = {
            "type": "execute",
            "code": "print('Hello from Python!')"
        }
        await websocket.send(json.dumps(request))
        response = json.loads(await websocket.recv())
        print("Output:", response["output"])
        if response["error"]:
            print("Error:", response["error"])

asyncio.run(execute_code())
```

### cURL (via wscat)

```bash
# Install wscat: npm install -g wscat
wscat -c ws://127.0.0.1:8080
# Then send:
{"type": "execute", "code": "print('Hello!')"}
```

## SMB Connection (Connecting to SMB Share)

WebSocket server supports connection to SMB (Samba/CIFS) share for working with files on remote servers.

### Requirements

**For Linux/Mac:**
```bash
brew install samba  # macOS
# or
sudo apt-get install samba-client  # Ubuntu/Debian
```

**For Windows:** SMB client is built into the system.

### Using lib:// Protocol

After successfully connecting to SMB share via `smb_connect` request, you can use special `lib://` protocol in DataCode scripts:

```
lib://share_name/path/to/file
```

Where `share_name` is the name of connected SMB share, and `path/to/file` is the path to file on the share.

### SMB Work Example

```python
import asyncio
import websockets
import json

async def smb_example():
    async with websockets.connect("ws://localhost:8899") as websocket:
        # 1. Connect to SMB
        connect_request = {
            "type": "smb_connect",
            "ip": "192.168.1.100",
            "login": "user",
            "password": "pass",
            "domain": "WORKGROUP",
            "share_name": "data"
        }
        await websocket.send(json.dumps(connect_request))
        response = json.loads(await websocket.recv())
        print("SMB Connect:", response)
        
        # 2. Execute DataCode script using SMB
        code = """
        local files = list_files(path("lib://data/reports"))
        for file in files do
            print("File:", file)
        next file
        """
        
        execute_request = {
            "type": "execute",
            "code": code
        }
        await websocket.send(json.dumps(execute_request))
        response = json.loads(await websocket.recv())
        print("Execute:", response)

asyncio.run(smb_example())
```

### Supported Operations

After connecting to SMB share, the following operations are available in DataCode:

- **list_files(path("lib://share_name/dir"))** - get list of files
- **read_file(path("lib://share_name/file.csv"))** - read file (CSV, XLSX, TXT supported)

For more details see `examples/08-websocket/README.md`.

## Features

1. **Session Isolation**: Each client gets its own interpreter. Variables and functions defined by one client are not visible to others.

2. **Output Interception**: All `print()` calls are intercepted and sent to client in `output` field.

3. **Error Handling**: Execution errors are returned in `error` field, and `success` is set to `false`.

4. **Multiline Code**: Multiline code execution is supported:

```json
{
  "type": "execute",
  "code": "global x = 10\nglobal y = 20\nprint('Sum:', x + y)"
}
```

5. **SMB Connections**: Each client has its own set of SMB connections, which are automatically closed when client disconnects.

## Web Client

Open file `examples/08-websocket/websocket_client_example.html` in browser for interactive WebSocket server testing.

## Security

⚠️ **Warning**: Current implementation does not include authentication or access restrictions. Do not use on public servers without additional protection!

## Limitations

- Interpreter is not thread-safe (`Send`), so each client is processed in a separate local task
- Variables and functions are not preserved between requests from one client (each request executes in the same interpreter, but state may be changed)
