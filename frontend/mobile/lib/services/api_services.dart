import 'dart:convert';

import '../widgets/api/toast_message.dart';
import 'package:http/http.dart' as http;

import '../helper/local_storage.dart';

class ApiServices {
  static var client = http.Client();

  static Future<String?> generateResponse1(String prompt, String model) async {
    var url = Uri.https("api.openai.com","/v1/completions");

    try{
      final response = await http.post(
        url,
        headers: {
          'Content-Type': 'application/json',
          "Authorization": "Bearer ${LocalStorage.getChatGptApiKey()}"
        },
        body: json.encode({
          "model": model,
          "prompt": prompt,
          "temperature": 0,
          "max_tokens": LocalStorage.getSelectedToken(),
          "top_p": 1,
          "frequency_penalty": 0.0,
          "presence_penalty": 0.0,
        }),
      );

      if(response.statusCode == 200){
        // Do something with the response
        Map<String, dynamic> newresponse = jsonDecode(utf8.decode(response.bodyBytes));

        return newresponse['choices'][0]['message']['content'];
      }else{
        ToastMessage.error(jsonDecode(response.body)["error"]["message"]);
      }

    }catch(e){
      ToastMessage.error(e.toString());
      return null;
    }
    return null;
  }


  static Future<String?> generateResponse2(String prompt) async {
    var url = Uri.https("api.openai.com", "/v1/chat/completions");

    try{
      final response = await http.post(
        url,
        headers: {
          'Content-Type': 'application/json',
          "Authorization": "Bearer ${LocalStorage.getChatGptApiKey()}"
        },
        body: json.encode({
          "model": "gpt-3.5-turbo",
          "messages": [
            {
              "role": "user",
              "content": prompt
            }
          ]
        }),
      );

      // print(response.statusCode);
      // print(response.body);
      if(response.statusCode == 200){
        // Do something with the response
        Map<String, dynamic> newresponse = jsonDecode(utf8.decode(response.bodyBytes));

        return newresponse['choices'][0]['message']['content'];
      }else{
        ToastMessage.error(jsonDecode(response.body)["error"]["message"]);
      }

    }catch (e){
      ToastMessage.error(e.toString());
      return null;
    }
    return null;

  }

  // static Future<AiTypeModel> getModels() async {
  //   var url = Uri.https("api.openai.com","/v1/models");
  //   final response = await http.get(
  //     url,
  //     headers: {
  //       'Content-Type': 'application/json',
  //       "Authorization": "Bearer ${LocalStorage.getChatGptApiKey()}"
  //     }
  //   );
  //
  //   // Do something with the response
  //   AiTypeModel newresponse = AiTypeModel.fromJson(jsonDecode(response.body));
  //
  //
  //   return newresponse;
  // }
}
