import os
import threading
import http.server
import socketserver
import webbrowser
import export_cytoscape

def serve_and_open():
    PORT = 8000
    
    # Previene l'errore "Address already in use" se lo script viene chiuso e riaperto velocemente
    class ReusableTCPServer(socketserver.TCPServer):
        allow_reuse_address = True

    Handler = http.server.SimpleHTTPRequestHandler

    try:
        httpd = ReusableTCPServer(("", PORT), Handler)
    except OSError:
        print(f"La porta {PORT} è già in uso. Apro semplicemente il browser...")
        webbrowser.open(f'http://localhost:{PORT}/visualize.html')
        return

    print(f"Server locale avviato all'indirizzo: http://localhost:{PORT}")
    
    def start_server():
        httpd.serve_forever()
        
    t = threading.Thread(target=start_server)
    t.daemon = True
    t.start()
    
    # Apre automaticamente la pagina nel browser predefinito
    webbrowser.open(f'http://localhost:{PORT}/visualize.html')
    
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
    print("1. Lettura di graph.json e generazione di cytoscape.json...")
    try:
        export_cytoscape.convert("graph.json", "cytoscape.json")
    except Exception as e:
        print(f"Errore durante l'esportazione: {e}")
        exit(1)
        
    print("2. Avvio del server web locale...")
    serve_and_open()
