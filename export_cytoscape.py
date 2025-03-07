import json
import sys

def qn_to_id(qn):
    if isinstance(qn, list):
        return "::".join(qn)
    return str(qn)

def convert(input_path, output_path):
    try:
        with open(input_path, 'r') as f:
            data = json.load(f)
    except FileNotFoundError:
        print(f"Errore: File {input_path} non trovato.")
        sys.exit(1)
    
    elements = []
    added_nodes = set()
    added_edges = set()
    global_edge_id = 0
    
    def add_node(id_str, label, type_str):
        if id_str not in added_nodes:
            added_nodes.add(id_str)
            node_data = {
                "id": id_str,
                "label": label,
                "type": type_str
            }
            elements.append({"data": node_data})

    def add_edge(source_id, target_id, label):
        nonlocal global_edge_id
        edge_sig = f"{source_id}->{target_id}:{label}"
        if edge_sig not in added_edges and source_id and target_id:
            added_edges.add(edge_sig)
            elements.append({
                "data": {
                    "id": f"e{global_edge_id}",
                    "source": source_id,
                    "target": target_id,
                    "label": label
                }
            })
            global_edge_id += 1

    # 1. Caricamento Nativo dei Nodi esportati dall'IR Rust
    for node_comp in data.get("nodes", []):
        if "Module" in node_comp:
            m = node_comp["Module"]
            name_parts = m.get("name", [])
            node_id = qn_to_id(name_parts)
            label = name_parts[-1] if name_parts else "root"
            add_node(node_id, label, "Module")
            
        elif "StructuredType" in node_comp:
            st = node_comp["StructuredType"]
            name_parts = st.get("name", [])
            node_id = qn_to_id(name_parts)
            label = name_parts[-1] if name_parts else "Unknown"
            add_node(node_id, label, st.get("kind", "Struct"))
            
        elif "Function" in node_comp:
            ff = node_comp["Function"]
            name_parts = ff.get("name", [])
            node_id = qn_to_id(name_parts)
            label = name_parts[-1] + "()" if name_parts else "()"
            add_node(node_id, label, "Function")

    # 2. Caricamento Nativo degli Archi estratti dal Type Resolver Rust
    for edge in data.get("edges", []):
        source_id = qn_to_id(edge.get("from"))
        target_id = qn_to_id(edge.get("to"))
        label = edge.get("kind", "")
        
        add_edge(source_id, target_id, label)
        
    # Validazione finale (per sicurezza rimuoviamo archi fantasma senza cancellare i nodi)
    valid_elements = [el for el in elements if el["data"].get("type")] # Tieni tutti i nodi
    for el in elements:
        d = el["data"]
        if "source" in d and "target" in d:
            if d["source"] in added_nodes and d["target"] in added_nodes:
                valid_elements.append(el)
    
    with open(output_path, 'w') as f:
        json.dump(valid_elements, f, indent=2)
    print(f"Grafo esportato con successo in {output_path} ({len(added_nodes)} nodi, {len(valid_elements) - len(added_nodes)} archi validi)")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Uso: python export_cytoscape.py <input.json> <output.json>")
        sys.exit(1)
    convert(sys.argv[1], sys.argv[2])
