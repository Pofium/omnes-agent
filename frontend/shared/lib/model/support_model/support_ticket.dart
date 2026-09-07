class SupportModel{
  SupportModel({
    required this.email,
    required this.name,
    required this.note,
  });

  String email;
  String name;
  String note;

  factory SupportModel.fromJson(Map<String, dynamic> json) => SupportModel(
    email: json["email"],
    name: json["name"],
    note: json["note"],
  );

  Map<String, dynamic> toJson() => {
    "email": email,
    "name": name,
    "note": note,
  };
}