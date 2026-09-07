// To parse this JSON data, do
//
//     final imageModel = imageModelFromJson(jsonString);

import 'dart:convert';

ImageModel imageModelFromJson(String str) =>
    ImageModel.fromJson(json.decode(str));

String imageModelToJson(ImageModel data) => json.encode(data.toJson());

class ImageModel {
  int created;
  List<Datum> data;

  ImageModel({
    required this.created,
    required this.data,
  });

  factory ImageModel.fromJson(Map<String, dynamic> json) => ImageModel(
        created: json["created"],
        data: List<Datum>.from(json["data"].map((x) => Datum.fromJson(x))),
      );

  Map<dynamic, dynamic> toJson() => {
        "created": created,
        "data": List<dynamic>.from(data.map((x) => x.toJson())),
      };
}

class Datum {
  String? url;

  Datum({
    this.url,
  });

  factory Datum.fromJson(Map<dynamic, dynamic> json) => Datum(
        url: json["url"],
      );

  Map<dynamic, dynamic> toJson() => {
        "url": url,
      };
}
