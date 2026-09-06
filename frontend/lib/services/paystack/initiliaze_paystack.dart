// import 'dart:convert';
//
// import 'package:omagent_front/utils/config.dart';
// import 'package:http/http.dart' as http;
//
// var url = Uri.parse('https://api.paystack.co/transaction/initialize');
//
// createAccessCode(skTest) async {
//   // skTest -> Secret key
//   Map<String, String> headers = {
//     'Content-Type': 'application/json',
//     'Accept': 'application/json',
//     'Authorization': 'Bearer $skTest'
//   };
//   Map data = {
//     "amount": ApiConfig.premiumSubscriptionFee.toInt(),
//     "email": "omagent@gmail.com"
//   };
//   String payload = json.encode(data);
//   http.Response response =
//       await http.post(url, headers: headers, body: payload);
//   return jsonDecode(response.body);
// }
