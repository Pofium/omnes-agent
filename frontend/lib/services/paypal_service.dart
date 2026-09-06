import 'dart:async';
import 'dart:convert' as convert;

import 'package:http/http.dart' as http;
import 'package:http_auth/http_auth.dart';

import '../helper/local_storage.dart';

class PaypalServices {
  String domain = "https://api.sandbox.paypal.com"; // for sandbox mode
 // String domain = "https://api.paypal.com"; // for production mode
 // String domain = "https://api-m.paypal.com"; // for production mode

  // change clientId and secret with your own, provided by paypal
  var clientId = LocalStorage.getPaypalClientId();
  var secret = LocalStorage.getPaypalSecret();

  // var clientId = "ASb5dpN8sZN07GqokcC2VxK2OBkZ6EnpEIzIDsei_Qo9uEoLNrN3NE2kY2g3W1LYkj5iW4IiA6k5fKB0";
  // var secret = "EH7YsgBpV-T9qz81yqUJRWYWzsK48Lah5V8skvHhprxzcGuBJdntpcRPeEjdXADDpicYbqfdpCObdwk3";

  // for getting the access token from Paypal
  Future<String> getAccessToken() async {

    try {
      var client = BasicAuthClient(clientId!, secret!);
      var response = await client.post(
          Uri.parse('$domain/v1/oauth2/token?grant_type=client_credentials'));
      if (response.statusCode == 200) {
        final body = convert.jsonDecode(response.body);
        return body["access_token"];
      }
      return '0';
    } catch (e) {
      rethrow;
    }
  }

  // for creating the payment request with Paypal
  Future<Map<String, String>> createPaypalPayment(
      transactions, accessToken) async {
    try {
      var response = await http.post(Uri.parse("$domain/v1/payments/payment"),
          body: convert.jsonEncode(transactions),
          headers: {
            "content-type": "application/json",
            'Authorization': 'Bearer $accessToken'
          });

      final body = convert.jsonDecode(response.body);
      if (response.statusCode == 201) {
        if (body["links"] != null && body["links"].length > 0) {
          List links = body["links"];

          String executeUrl = "";
          String approvalUrl = "";
          final item = links.firstWhere((o) => o["rel"] == "approval_url",
              orElse: () => null);
          if (item != null) {
            approvalUrl = item["href"];
          }
          final item1 = links.firstWhere((o) => o["rel"] == "execute",
              orElse: () => null);
          if (item1 != null) {
            executeUrl = item1["href"];
          }
          return {"executeUrl": executeUrl, "approvalUrl": approvalUrl};
        }
        return {"executeUrl": '', "approvalUrl": ''};
      } else {
        throw Exception(body["message"]);
      }
    } catch (e) {
      rethrow;
    }
  }

  // for executing the payment transaction
  Future<String> executePayment(url, payerId, accessToken) async {
    try {
      var response = await http.post(Uri.parse(url),
          body: convert.jsonEncode({"payer_id": payerId}),
          headers: {
            "content-type": "application/json",
            'Authorization': 'Bearer $accessToken'
          });

      final body = convert.jsonDecode(response.body);
      if (response.statusCode == 200) {
        return body["id"];
      }
      return '0';
    } catch (e) {
      rethrow;
    }
  }
}
