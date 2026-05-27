import http.server
import json
import os
import socketserver
import threading
import webbrowser

# Cambiamo directory nella root del progetto per far sì che il webserver possa servire
# sia il file HTML in visualizer/ sia i json in tests/outputs/
script_dir = os.path.dirname(os.path.abspath(__file__))
project_root = os.path.dirname(script_dir)
os.chdir(project_root)

OUTPUT_DIR = "tests/outputs"

class CustomHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/api/list_graphs':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            
            # Trova tutti i file cyto_*.json generati
            files = []
            if os.path.exists(OUTPUT_DIR):
                for f in os.listdir(OUTPUT_DIR):
                    if f.startswith("cyto_") and f.endswith(".json"):
                        files.append(os.path.join(OUTPUT_DIR, f))
            
            self.wfile.write(json.dumps(files).encode())
        else:
            super().do_GET()

def serve_and_open():
    PORT = 8000
    
    class ReusableTCPServer(socketserver.TCPServer):
        allow_reuse_address = True

    try:
        httpd = ReusableTCPServer(("", PORT), CustomHandler)
    except OSError:
        print(f"La porta {PORT} è già in uso. Apro semplicemente il browser...")
        webbrowser.open(f'http://localhost:{PORT}/visualizer/visualize.html')
        return

    print(f"Server locale avviato all'indirizzo: http://localhost:{PORT}")
    
    def start_server():
        httpd.serve_forever()
        
    t = threading.Thread(target=start_server)
    t.daemon = True
    t.start()
    
    webbrowser.open(f'http://localhost:{PORT}/visualizer/visualize.html')
    
    print("\n[ SERVER ATTIVO ]")
    print("Premi INVIO o CTRL+C nel terminale per fermare il server ed uscire...")
    try:
        input()
    except KeyboardInterrupt:
        pass
    finally:
        print("Arresto del server in corso...")
        httpd.shutdown()

if __name__ == "__main__":
    print("1. Avvio del server web locale...")
    serve_and_open()
