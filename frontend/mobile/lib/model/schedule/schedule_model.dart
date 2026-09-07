class ScheduleModel {
  String title;
  String description;
  String scheduleType;
  String time;
  String date;
  int id;
  List<String> weeks;

  ScheduleModel({
    required this.title,
    required this.description,
    required this.scheduleType,
    required this.time,
    required this.date,
    required this.id,
    required this.weeks,
  });

  factory ScheduleModel.fromJson(Map<String, dynamic> json) {
    return ScheduleModel(
      title: json['title'] as String,
      description: json['description'] as String,
      scheduleType: json['scheduleType'] as String,
      time: json['time'] as String,
      date: json['date'] as String,
      id: json['id'] as int,
      weeks: (json['weeks'] as List<dynamic>).map((e) => e as String).toList(),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'title': title,
      'description': description,
      'scheduleType': scheduleType,
      'time': time,
      'date': date,
      'id': id,
      'weeks': weeks,
    };
  }
}