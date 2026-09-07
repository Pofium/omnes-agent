class AiTypeModel {
  AiTypeModel({
    required this.mainData,
  });
  late final List<Data> mainData;

  AiTypeModel.fromJson(Map<String, dynamic> json){
    mainData = List.from(json['data']).map((e)=>Data.fromJson(e)).toList();
  }

  Map<String, dynamic> toJson() {
    final data = <String, dynamic>{};
    data['data'] = mainData.map((e)=>e.toJson()).toList();
    return data;
  }
}

class Data {
  Data({
    required this.id,
    // required this.object,
    // required this.created,
    // required this.ownedBy,
    // // required this.permission,
    // required this.root,
  });
  late final dynamic id;
  // late final String object;
  // late final int created;
  // late final String ownedBy;
  // // late final List<Permission> permission;
  // late final String root;

  Data.fromJson(Map<String, dynamic> json){
    id = json['id'] ?? '';
    // object = json['object'];
    // created = json['created'];
    // ownedBy = json['owned_by'];
    // // permission = List.from(json['permission']).map((e)=>Permission.fromJson(e)).toList();
    // root = json['root'];
  }

  Map<String, dynamic> toJson() {
    final data = <String, dynamic>{};
    data['id'] = id ;
    // data['object'] = object;
    // data['created'] = created;
    // data['owned_by'] = ownedBy;
    // // data['permission'] = permission.map((e)=>e.toJson()).toList();
    // data['root'] = root;
    return data;
  }
}
//
// class Permission {
//   Permission({
//     required this.id,
//     required this.object,
//     required this.created,
//     required this.allowCreateEngine,
//     required this.allowSampling,
//     required this.allowLogprobs,
//     required this.allowSearchIndices,
//     required this.allowView,
//     required this.allowFineTuning,
//     required this.organization,
//     required this.isBlocking,
//   });
//   late final String id;
//   late final String object;
//   late final int created;
//   late final bool allowCreateEngine;
//   late final bool allowSampling;
//   late final bool allowLogprobs;
//   late final bool allowSearchIndices;
//   late final bool allowView;
//   late final bool allowFineTuning;
//   late final String organization;
//   late final bool isBlocking;
//
//   Permission.fromJson(Map<String, dynamic> json){
//     id = json['id'];
//     object = json['object'];
//     created = json['created'];
//     allowCreateEngine = json['allow_create_engine'];
//     allowSampling = json['allow_sampling'];
//     allowLogprobs = json['allow_logprobs'];
//     allowSearchIndices = json['allow_search_indices'];
//     allowView = json['allow_view'];
//     allowFineTuning = json['allow_fine_tuning'];
//     organization = json['organization'];
//     isBlocking = json['is_blocking'];
//   }
//
//   Map<String, dynamic> toJson() {
//     final data = <String, dynamic>{};
//     data['id'] = id;
//     data['object'] = object;
//     data['created'] = created;
//     data['allow_create_engine'] = allowCreateEngine;
//     data['allow_sampling'] = allowSampling;
//     data['allow_logprobs'] = allowLogprobs;
//     data['allow_search_indices'] = allowSearchIndices;
//     data['allow_view'] = allowView;
//     data['allow_fine_tuning'] = allowFineTuning;
//     data['organization'] = organization;
//     data['is_blocking'] = isBlocking;
//     return data;
//   }
// }