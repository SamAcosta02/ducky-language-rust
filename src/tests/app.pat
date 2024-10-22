program patito_test123;
  vars age, ocurrences: int; height_meters: float;

  void create_user ( user_height: float, confirm_age: int ) {
    total_height = user_height - imaginary_value_10 + 1;
    real_age = confirm_age + 0;
  };

  void delete_user ( user_height: float, confirm_age: int ) {
    total_height = 0;
    real_age = 0;
  };
  
  begin 
  {
    input_user_height = height_meters;
    ocurrences = ocurrences + 1;
    input_age = age;

    create_user(input_user_height, input_age - 1);
    delete_user(input_user_height, input_age);

    while (input_age < 25) do {
      print("The age is now: ", input_age);
      input_age = input_age - 1;
      if (input_age < 30) {
        print("The person's age is now very young");
      };
    };
  }
  end
