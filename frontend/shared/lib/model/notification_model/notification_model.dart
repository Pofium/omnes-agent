class NotificationModel{
  final String title, body, url;

  NotificationModel({
    required this.title,
    required this.body,
    this.url = '',
  });

  factory NotificationModel.fromJson(Map<String, dynamic> json) => NotificationModel(
    title: json["title"],
    body: json["body"],
    url: json["url"],
  );

  Map<String, dynamic> toJson() => {
    "title": title,
    "body": body,
    "url": url,
  };
}