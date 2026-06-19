const style = [
	{
		selector: 'node',
		style: {
			'label': 'data(label)',
			'text-valign': 'center',
			'text-halign': 'center',
			'color': '#fff',
			'text-outline-width': 2,
			'text-outline-color': '#444',
			'background-color': '#555',
			'font-size': '12px',
			'width': 'label',
			'height': 'label',
			'padding': '10px',
			'shape': 'round-rectangle',
			'transition-property': 'opacity',
			'transition-duration': '0.3s'
		}
	},
	{
		selector: ':parent',
		style: {
			'text-valign': 'top',
			'text-halign': 'center',
			'background-opacity': 0.15,
			'border-width': 2,
			'border-style': 'dashed',
			'padding': '15px',
			'color': '#ffffff',
			'text-outline-width': 0,
			'font-size': '16px',
			'font-weight': 'bold',
			'text-margin-y': -8
		}
	},
	{
		selector: ':parent[type = "Module"]',
		style: {
			'background-color': '#d35400',
			'border-color': '#d35400'
		}
	},
	{
		selector: ':parent[type = "Class"], :parent[type = "Struct"]',
		style: {
			'background-color': '#2980b9',
			'border-color': '#2980b9',
            'border-style': 'solid'
		}
	},
	{
		selector: ':parent[type = "Interface"], :parent[type = "Trait"]',
		style: {
			'background-color': '#27ae60',
			'border-color': '#2ecc71',
            'border-style': 'solid'
		}
	},
	{
		selector: 'node[type = "Module"]',
		style: {
			'background-color': '#d35400',
			'border-color': '#d35400'
		}
	},
	{
		selector: 'node[type = "Struct"], node[type = "Class"]',
		style: {
			'background-color': '#2980b9'
		}
	},
	{
		selector: 'node[type = "Trait"], node[type = "Interface"]',
		style: {
			'background-color': '#27ae60',
			'border-style': 'solid',
			'border-width': 2,
			'border-color': '#2ecc71'
		}
	},
	{
		selector: 'node[type = "Function"]',
		style: {
			'background-color': '#8e44ad',
			'shape': 'ellipse',
			'padding': '6px'
		}
	},
	{
		selector: 'node[type = "External"]',
		style: {
			'background-color': '#34495e',
			'opacity': 0.7,
			'border-width': 1,
			'border-color': '#95a5a6'
		}
	},
	{
		selector: 'edge',
		style: {
			'width': 2,
			'line-color': '#aaaaaa',
			'target-arrow-color': '#aaaaaa',
			'target-arrow-shape': 'triangle',
			'curve-style': 'bezier',
			'label': 'data(label)',
			'font-size': '9px',
			'color': '#ccc',
			'text-rotation': 'autorotate',
			'text-background-opacity': 0.8,
			'text-background-color': '#1e1e1e',
			'text-background-padding': '3px',
			'transition-property': 'opacity',
			'transition-duration': '0.3s'
		}
	},
	{
		selector: 'edge[label = "IsA"]',
		style: {
			'width': 3,
			'line-style': 'solid',
			'line-color': '#e74c3c',
			'target-arrow-color': '#e74c3c',
			'target-arrow-shape': 'triangle'
		}
	},
	{
		selector: 'edge[label = "Calls"]',
		style: {
			'line-color': '#2ecc71',
			'target-arrow-color': '#2ecc71',
			'width': 2
		}
	},
	{
		selector: 'edge[label = "Instantiates"]',
		style: {
			'line-color': '#3498db',
			'target-arrow-color': '#3498db',
			'line-style': 'dashed',
			'width': 2
		}
	},
	{
		selector: 'edge[label ^= "Uses"]',
		style: {
			'line-color': '#95a5a6',
			'target-arrow-color': '#95a5a6',
			'line-style': 'dotted',
			'opacity': 0.7
		}
	},
	{
		selector: '.dimmed',
		style: {
			'opacity': 0.15
		}
	}
]

export default style;