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
			'shape': 'round-rectangle'
		}
	},
	{
		selector: 'node[type = "Module"]',
		style: {
			'background-color': '#d35400',
			'shape': 'hexagon'
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
			'border-style': 'dashed',
			'border-width': 2,
			'border-color': '#fff'
		}
	},
	{
		selector: 'node[type = "Enum"]',
		style: {
			'background-color': '#f1c40f',
			'color': '#000',
			'text-outline-color': '#f1c40f'
		}
	},
	{
		selector: 'node[type = "Function"]',
		style: {
			'background-color': '#8e44ad',
			'shape': 'ellipse'
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
		selector: 'node[type = "ImplBlock"]',
		style: {
			'background-color': '#7f8c8d',
			'shape': 'rectangle'
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
			'font-size': '8px',
			'color': '#ccc',
			'text-rotation': 'autorotate',
			'text-background-opacity': 0.8,
			'text-background-color': '#1e1e1e',
			'text-background-padding': '2px'
		}
	},
	{
		selector: 'edge[label = "Inherits"], edge[label = "Implements"]',
		style: {
			'width': 3,
			'line-style': 'dashed',
			'line-color': '#f39c12',
			'target-arrow-color': '#f39c12',
			'target-arrow-shape': 'triangle-tee'
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
			'line-color': '#ccd1d1',
			'target-arrow-color': '#ccd1d1',
			'line-style': 'dotted',
			'opacity': 0.7
		}
	},
	{
		selector: 'edge[label = "NestedIn"], edge[label = "ModuleContainment"]',
		style: {
			'line-color': '#707b7c',
			'target-arrow-color': '#707b7c',
			'target-arrow-shape': 'circle',
			'line-style': 'dotted'
		}
	}
]

export default style;
