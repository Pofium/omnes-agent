/// Structured citation linking an agent response claim to a file, AST node, or memory.
class GroundingCitation {
  final String citationId;
  final String sourceType; // 'file', 'ast_node', 'memory'
  final String reference;
  final String title;
  final String? snippet;

  const GroundingCitation({
    required this.citationId,
    required this.sourceType,
    required this.reference,
    required this.title,
    this.snippet,
  });

  factory GroundingCitation.fromJson(Map<String, dynamic> json) {
    return GroundingCitation(
      citationId: json['citation_id'] as String? ?? json['id'] as String? ?? '',
      sourceType: json['source_type'] as String? ?? 'file',
      reference: json['reference'] as String? ?? '',
      title: json['title'] as String? ?? json['reference'] as String? ?? '',
      snippet: json['snippet'] as String?,
    );
  }

  Map<String, dynamic> toJson() => {
    'citation_id': citationId,
    'source_type': sourceType,
    'reference': reference,
    'title': title,
    if (snippet != null) 'snippet': snippet,
  };
}
