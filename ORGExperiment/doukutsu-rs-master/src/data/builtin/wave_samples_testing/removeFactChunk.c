#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stddef.h>


int main(int argc, void ** argv)
{
    //1 for name, 2 for input file/output file

    if(argc != 3)
    {
        printf("incorrect argument count!\n");
        return EXIT_FAILURE;
    }
    //read file size
    FILE* fp = fopen(argv[1], "rb");
    if(fp == NULL)
    {
        printf("invalid file path!\n");
        return EXIT_FAILURE;
    }
    printf("reading from %s\n", argv[1]);

    fseek(fp, 0, SEEK_END);
    int fsize = ftell(fp);
    fseek(fp, 0, SEEK_SET);
    //make buffer to hold file
    unsigned char* buffer =  malloc(fsize * sizeof(char));
    //read into buffer
    fread(buffer, fsize, 1, fp);

    //size of the file minus the 8 bytes at the front
    int loadsize = fsize - 8;
    printf("file is  %x bytes long\n", loadsize);


    //find data offset
    unsigned char* looker = "data"; //thing to look for
    short size_comapre = 4;
    unsigned char* strptr = NULL; //strstr(buffer, "data"); //can't do this, strstr doesn't play ball with binary


    //Assume data_len is length of the data from which text is to be found and data is pointer (char*) to the start of it.
    unsigned int count = 0;
    for(; count < fsize - size_comapre; ++count)
    {
        if(!memcmp(looker, buffer + count, size_comapre))
        {
            strptr = buffer + count; //update pointer
            printf("data chunk found at %x\n", count);
            break;
        }
    }


    if(strptr == NULL)
    {
        printf("data not found!\n");
        printf("%c%c%c%c", buffer[0x32], buffer[0x33], buffer[0x34], buffer[0x35]);
        free(buffer);
        return EXIT_FAILURE;
    }
    printf("found data at slot %d\n", count);


    //read the size of the data chunk (because really crappy wavs lie about this number)
    unsigned char* aa = strptr + 0x4;
    unsigned int data_ch_size = (aa[3] << 24) | (aa[2] << 16) | (aa[1] << 8) | aa[0];

    //new total file size
    unsigned int newSize = 0x2C + data_ch_size;
    printf("Old filesize: %x, New filesize: %x\n", fsize, newSize);


    //how much we KNOW we are deleting (the length of everything between the fmt and data chunks)
    unsigned int deleteSize = count - 0x24;


    //reopen the file to write out
    fclose(fp);
    fp = fopen(argv[2], "wb");
    if(fp == NULL)
    {
        printf("could not open file for writing!\n");
        free(buffer);
        return EXIT_FAILURE;
    }
    printf("writing to %s\n", argv[2]);


    //write header up to the end of the required section

    //write start
    fwrite(buffer, 1, 0x4, fp);
    //write new file size
    for (unsigned int i = 0; i < 4; ++i)
		fputc((newSize - 0x8) >> (8 * i), fp);
    //write part of the fmt chunk
    fwrite(buffer + 8, 0x10 - 8, 1, fp);

    //write new fmt chunk size (we know this will be 0x16 because there is nothing else anymore)
    for (unsigned int i = 0; i < 4; ++i)
		fputc(0x10 >> (8 * i), fp);

    //write everything else up to the data chunk
    fwrite(buffer + 0x14, 0x24 - 0x14, 1, fp);

    //write out everything after this
    int deltaSize = fsize - (newSize + deleteSize);
    if(deltaSize)
        printf("warning! sizes do not match!\n");
    

    fwrite(strptr, 1, fsize - count - deltaSize, fp);

    free(buffer);
    fclose(fp);

    return EXIT_SUCCESS;
}