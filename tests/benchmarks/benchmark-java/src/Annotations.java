package com.example;

@MyAnnotation
public class AnnotatedClass {
    @Inject
    public void myMethod() {}
}

@interface MyAnnotation {}
@interface Inject {}
